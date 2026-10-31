# Development commands. `just` lists them; the agent harness uses the same entry points.
import 'cardinal.just'
set allow-duplicate-recipes

default:
    @just --list

# Format, lint (warning-free), test, then check the specs; prints nothing on success, since the agent reads all of it.
# Strict: this repository ships the managed tests, so their citations must resolve to a spec here (consumers stay
# lenient).
check: fmt clippy test
    @just check-specs --strict

fmt:
    @cargo fmt --all

clippy:
    @cargo clippy -q --workspace --all-targets -- -D warnings

# The output of the tests, only when they fail.
test:
    #!/usr/bin/env sh
    out=$(cargo test -q --workspace 2>&1) || { printf '%s\n' "$out"; exit 1; }

# Install the `cardinal` binary to ~/.cargo/bin (`--force`: over the installed one, even of the same version).
install:
    cargo install --force --path crates/cardinal --locked

# This repository is its own first consumer: check it with the binary just built.
check-cardinal:
    cargo run -q -p cardinal -- check

# The spec checker of the working tree, whatever `cardinal` is installed (CI installs none for this repository).
check-specs *args:
    @cargo run -q -p cardinal -- run check-specs {{args}}

# Write `version` into the crate and its lock (the `release_version_command` of `just release`).
set-version version:
    sed -i.bak '/^\[package\]/,/^\[/s/^version = ".*"/version = "{{version}}"/' crates/cardinal/Cargo.toml && rm crates/cardinal/Cargo.toml.bak
    cargo update --workspace --offline -q

# Run the issue of `fixtures/replay/` headless `runs` times against the Cardinal revision `rev`, each in a fresh copy
# kept under /tmp, then grade it: one tab-separated line per run. Billed to the user's plan; `claude` names another
# command for a dry run (it receives the arguments of `claude -p` and prints its JSON result).
replay rev="HEAD" runs="1" claude="claude":
    #!/usr/bin/env bash
    set -euo pipefail
    repo=$PWD
    commit=$(git rev-parse --verify "{{rev}}^{commit}")
    issue=issues/count_the_frequency_of_each_word.md
    printf 'run\tdir\tsession\tsubtype\tduration_s\tcost_usd\tturns\tinput_tokens\toutput_tokens\tcache_read_tokens\tcache_creation_tokens\tcommits\treviews\tgraded\n'
    for run in $(seq 1 "{{runs}}"); do
        dir=$(mktemp -d /tmp/cardinal-replay.XXXXXX)/replay-fixture
        mkdir "$dir"
        tar -C fixtures/replay --exclude=./grade --exclude=./target -cf - . | tar -C "$dir" -xf -
        (
            cd "$dir"
            git init -q -b main
            cardinal init "$repo/.git" --rev "$commit" >/dev/null
            # The setup is committed before the review gate exists, then a second sync installs it.
            cardinal sync >/dev/null
            rm .git/hooks/pre-commit
            git add -- . && git commit -q -m 'chore: set up the replay fixture'
            cardinal sync >/dev/null
            test -x .git/hooks/pre-commit
            sed -i "s/^- Base commit: BASE$/- Base commit: $(git rev-parse HEAD)/" "$issue"
            git commit -q -m 'docs: record the base commit' -- "$issue"
        ) >&2
        base=$(git -C "$dir" rev-parse HEAD)
        # Only the review lines the run adds count, not those the handed issue already holds.
        handed=$(grep -ci 'code review' "$dir/$issue" || true)
        start=$(date +%s)
        (cd "$dir" && "{{claude}}" -p "Work the issue $issue." --output-format json --permission-mode auto \
            --max-budget-usd 5) >"$dir.json" || true
        elapsed=$(( $(date +%s) - start ))
        commits=$(git -C "$dir" rev-list --count "$base..HEAD")
        reviews=$(cat "$dir/$issue" "$dir/issues/done/${issue#issues/}" 2>/dev/null | grep -ci 'code review' || true)
        reviews=$(( reviews - handed ))
        mkdir -p "$dir/tests" && cp fixtures/replay/grade/grade.rs "$dir/tests/replay_grade.rs"
        graded=fail
        (cd "$dir" && cargo test -q >/dev/null 2>&1) && graded=pass
        jq -er --arg run "$run" --arg dir "$dir" --arg elapsed "$elapsed" --arg commits "$commits" \
            --arg reviews "$reviews" --arg graded "$graded" \
            '[$run, $dir, .session_id, .subtype, $elapsed, .total_cost_usd, .num_turns, .usage.input_tokens,
              .usage.output_tokens, .usage.cache_read_input_tokens, .usage.cache_creation_input_tokens, $commits,
              $reviews, $graded] | map(. // "" | tostring) | join("\t")' "$dir.json" 2>/dev/null \
            || printf '%s\t%s\t\tunreadable\t%s\t\t\t\t\t\t\t%s\t%s\t%s\n' "$run" "$dir" "$elapsed" "$commits" "$reviews" "$graded"
    done

# Bring this repository in step with its own `files/` and `seed/`.
sync *args:
    cargo run -q -p cardinal -- sync {{args}}
