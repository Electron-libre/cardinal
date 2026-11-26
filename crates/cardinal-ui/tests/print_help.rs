//! Behavior test for the help line of `cardinal-ui --print` (SUA-15): the keys the selection the screen starts on
//! allows, against throwaway git repositories, a fake `cardinal` and a fake `zellij`.

mod common;

use common::{git, ok, project, ui, ui_in, QUEUE, ROADMAP};

/// The help line, the last line of `out`.
fn help(out: &str) -> &str {
    out.lines().last().unwrap_or_default()
}

/// A queue of one issue `b`, taken by a claim in the state `state` whose session is `alive`.
fn claimed(state: &str, alive: bool) -> String {
    format!(
        r#"{{"version":1,"next":null,"issues":[{{"stem":"b","title":"Title b","priority":"high","why":"","milestone":null,"depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":{{"by":"claim","state":"{state}","worktree":"WS","branch":"b","session_alive":{alive}}},"stale":false}}]}}"#
    )
}

/// A roadmap of the milestone `New`, `done`, and the `release --json` that lists it (REL-15).
const DONE: &str = r#"{"version":1,"milestones":[{"name":"New","state":"done","on_hold":null,"released":null,"document":"ROADMAP.md","issues":[{"path":"issues/done/a.md","stem":"a","open":false}]}]}"#;
const RELEASE: &str = r#"{"version":1,"proposed":"0.3.0","prepared":false,"last":"0.2.0","commits":2,"milestones":[{"name":"New","document":"ROADMAP.md"}]}"#;

// spec: SUA-15
#[test]
fn outside_zellij_a_ready_issue_offers_only_the_keys_that_always_work() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(
        help(&out),
        ": commands  ↑↓ select  tab panel  q quit  i summary",
        "{out}"
    );
}

// spec: SUA-15, SUA-16
#[test]
fn in_zellij_a_ready_issue_nothing_takes_offers_its_start() {
    let p = project();
    let cardinal = p.cardinal(&ok(QUEUE), &ok(ROADMAP));
    let zellij = p.zellij("[]", 0);
    let (_, out, _) = ui_in(&p.repo, &cardinal, Some(&zellij), &["--print"]);
    assert_eq!(
        help(&out),
        ": commands  ↑↓ select  tab panel  q quit  i summary  s start",
        "{out}"
    );
}

// spec: SUA-15
#[test]
fn a_working_claim_whose_session_ended_offers_its_start_and_a_live_one_nothing() {
    let p = project();
    let zellij = p.zellij("[]", 0);
    for (alive, keys) in [(false, "  s start"), (true, "")] {
        let cardinal = p.cardinal(&ok(&claimed("working", alive)), &ok(ROADMAP));
        let (_, out, _) = ui_in(&p.repo, &cardinal, Some(&zellij), &["--print"]);
        assert_eq!(
            help(&out),
            format!(": commands  ↑↓ select  tab panel  q quit  i summary{keys}"),
            "{out}"
        );
    }
}

// spec: SUA-15
#[test]
fn a_merge_pending_issue_offers_its_merge_and_the_keys_of_the_whole_project_stay_off_the_line() {
    let p = project();
    std::fs::create_dir_all(p.repo.join("issues/done")).unwrap();
    std::fs::write(p.repo.join("issues/done/ws.md"), "closed\n").unwrap();
    git(&p.repo, &["add", "issues/done/ws.md"]);
    git(&p.repo, &["commit", "-qm", "close ws", "--no-verify"]);
    let cardinal = p.cardinal_releasing(
        &ok(&claimed("merge pending", false)),
        &ok(DONE),
        Some(&ok(RELEASE)),
    );
    let zellij = p.zellij("[]", 0);
    let (_, out, _) = ui_in(&p.repo, &cardinal, Some(&zellij), &["--print"]);
    assert_eq!(
        help(&out),
        ": commands  ↑↓ select  tab panel  q quit  i summary  f merge",
        "{out}"
    );
    // the release, the end of the merged worktrees and the review of the priorities are in the palette (SUP-1)
    let (_, out, _) = ui(&p.repo, &cardinal, &["--print"]);
    assert_eq!(
        help(&out),
        ": commands  ↑↓ select  tab panel  q quit  i summary  f merge",
        "{out}"
    );
}

// spec: SUA-15
#[test]
fn in_zellij_an_empty_queue_offers_the_summary_of_its_milestone_heading() {
    let p = project();
    let empty = r#"{"version":1,"next":null,"issues":[]}"#;
    let cardinal = p.cardinal(&ok(empty), &ok(ROADMAP));
    let zellij = p.zellij("[]", 0);
    let (_, out, _) = ui_in(&p.repo, &cardinal, Some(&zellij), &["--print"]);
    // the selection rests on the heading of `One`, whose summary `i` unfolds (SUQ-6)
    assert_eq!(
        help(&out),
        ": commands  ↑↓ select  tab panel  q quit  i summary",
        "{out}"
    );
}

// spec: STU-14, SUA-15
#[test]
fn print_starts_on_the_issue_next_issue_offers() {
    let p = project();
    // `a`, taken, comes first; `b` is the one `next-issue` offers
    let queue = r#"{"version":1,"next":"b","issues":[
{"stem":"a","title":"Title a","priority":"high","why":"","milestone":"One","depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":{"by":"claim","state":"working","worktree":"WS","branch":"ws","session_alive":true},"stale":false},
{"stem":"b","title":"Title b","priority":"low","why":"","milestone":"One","depends_on":[],"state":"ready","blocked_by":[],"hold":null,"taken":null,"stale":false}]}"#;
    let cardinal = p.cardinal(&ok(queue), &ok(ROADMAP));
    let zellij = p.zellij("[]", 0);
    let (_, out, _) = ui_in(&p.repo, &cardinal, Some(&zellij), &["--print"]);
    assert_eq!(
        help(&out),
        ": commands  ↑↓ select  tab panel  q quit  i summary  s start",
        "{out}"
    );
}
