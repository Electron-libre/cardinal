//! The threads of the screen whose panic ends as a failure (SUS-7, SUS-8): a task, a measure and a notification run
//! on a named thread under `catch_unwind`, their panic turned into the failed line each one reports, and the panic
//! hook passes over the first panic of these threads, so it prints nothing over the screen that goes on.

/// A thread whose panic `spawn` reports as a failure.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Caught {
    Task,
    Measure,
    Notification,
}

/// Every thread of `Caught`, in the order of its variants.
const CAUGHT: [Caught; 3] = [Caught::Task, Caught::Measure, Caught::Notification];

impl Caught {
    /// The name of its threads, which the hook of `pass_over_caught` passes over.
    pub fn name(self) -> &'static str {
        match self {
            Caught::Task => "cardinal-ui task",
            Caught::Measure => "cardinal-ui measure",
            Caught::Notification => "cardinal-ui notification",
        }
    }

    /// The line of a panic of its thread, as the screen shows it (SUS-7, SUS-8): a notification prefixed `notify: `
    /// like its other failures (STU-30).
    fn panicked(self, panic: &(dyn std::any::Any + Send)) -> String {
        let what = match self {
            Caught::Task => "the task",
            Caught::Measure => "the measure",
            Caught::Notification => "notify: the notification",
        };
        format!("{what} panicked: {}", message(panic))
    }
}

/// Runs `job` on a thread `of` names and gives its result to `end`, a panic as the failed line `Caught::panicked`
/// words; an error when the thread could not start.
pub fn spawn<T: 'static>(
    of: Caught,
    job: impl FnOnce() -> Result<T, String> + std::panic::UnwindSafe + Send + 'static,
    end: impl FnOnce(Result<T, String>) + Send + 'static,
) -> std::io::Result<()> {
    let body = move || {
        let result =
            std::panic::catch_unwind(job).unwrap_or_else(|panic| Err(of.panicked(&*panic)));
        end(result);
    };
    std::thread::Builder::new()
        .name(of.name().into())
        .spawn(body)
        .map(drop)
}

/// The message of a panic: the text `panic!` was given, else a placeholder.
fn message(panic: &(dyn std::any::Any + Send)) -> &str {
    panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("no message")
}

thread_local! {
    /// Whether the hook of `pass_over_caught` already passed over a panic of this thread.
    static PASSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Makes the panic hook pass over the first panic of a thread of `Caught`, which `spawn` reports: the hook Ratatui
/// set would give the terminal back and print over the screen that goes on (SUS-7, SUS-8). A second panic of that
/// thread (one during the unwinding of the first, which aborts the process) reaches it, so the terminal is given
/// back. A thread these threads would start is none of them: they start none.
pub fn pass_over_caught() {
    let before = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let current = std::thread::current();
        let caught = CAUGHT.iter().any(|c| current.name() == Some(c.name()));
        let first_of_caught = caught && !PASSED.replace(true);
        if !first_of_caught {
            before(info);
        }
    }));
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::{mpsc, Arc, Mutex, MutexGuard, PoisonError};
    use std::time::Duration;

    /// A panic hook of the process.
    type Hook = dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync + 'static;

    /// The panic hook is global to the process: the tests that panic on a caught thread, or that set it, take turns.
    static HOOK: Mutex<()> = Mutex::new(());

    /// The turn of a test that panics on a caught thread or sets the hook.
    pub fn turn() -> MutexGuard<'static, ()> {
        HOOK.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The result `spawn` gives of a job that panics with `boom` on a thread of `of`.
    fn panicking(of: Caught) -> Result<(), String> {
        let (send, ended) = mpsc::channel();
        let end = move |result| send.send(result).unwrap();
        spawn(of, || -> Result<(), String> { panic!("boom") }, end).unwrap();
        ended.recv_timeout(Duration::from_secs(5)).unwrap()
    }

    // spec: SUS-7 (internal: the panic of a task of a screen that needs a real terminal)
    // spec: SUS-8 (internal: the panic of a measure or a notification of a screen that needs a real terminal)
    #[test]
    fn a_caught_thread_that_panics_gives_its_panic_as_a_failure() {
        let _turn = turn();
        for (of, line) in [
            (Caught::Task, "the task panicked: boom"),
            (Caught::Measure, "the measure panicked: boom"),
            (
                Caught::Notification,
                "notify: the notification panicked: boom",
            ),
        ] {
            assert_eq!(panicking(of), Err(line.to_string()));
        }
    }

    /// The names of the threads whose panic reached the hook `pass_over_caught` wraps, while `body` runs; the hook of
    /// the process given back after.
    fn reaching_the_terminal(body: impl FnOnce()) -> Vec<Option<String>> {
        let _turn = turn();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&seen);
        let original: Arc<Hook> = Arc::from(std::panic::take_hook());
        let inner = Arc::clone(&original);
        std::panic::set_hook(Box::new(move |info| {
            let name = std::thread::current().name().map(String::from);
            record.lock().unwrap().push(name);
            inner(info);
        }));
        pass_over_caught();
        body();
        drop(std::panic::take_hook());
        std::panic::set_hook(Box::new(move |info| original(info)));
        let seen = seen.lock().unwrap().clone();
        seen
    }

    // spec: SUS-7 (internal: the panic hook of a screen that needs a real terminal)
    // spec: SUS-8 (internal: the panic hook of a screen that needs a real terminal)
    #[test]
    fn the_panic_of_a_caught_thread_reaches_no_hook_of_the_terminal() {
        let seen = reaching_the_terminal(|| {
            for of in CAUGHT {
                assert!(panicking(of).is_err());
            }
            let named = std::thread::Builder::new().name("other".into());
            named.spawn(|| panic!("seen")).unwrap().join().unwrap_err();
        });
        assert!(seen.contains(&Some("other".to_string())), "{seen:?}");
        for of in CAUGHT {
            assert!(!seen.contains(&Some(of.name().to_string())), "{seen:?}");
        }
    }

    // spec: SUS-7 (internal: a second panic of a task aborts the process, which no test survives)
    #[test]
    fn a_second_panic_of_a_caught_thread_reaches_the_hook_of_the_terminal() {
        let seen = reaching_the_terminal(|| {
            let task = std::thread::Builder::new().name(Caught::Task.name().into());
            let twice = || {
                for _ in 0..2 {
                    let _ = std::panic::catch_unwind(|| panic!("again"));
                }
            };
            task.spawn(twice).unwrap().join().unwrap();
        });
        let task = Some(Caught::Task.name().to_string());
        assert_eq!(seen.iter().filter(|n| **n == task).count(), 1, "{seen:?}");
    }
}
