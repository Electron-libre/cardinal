# Development commands. `just` lists them; the agent harness uses the same entry points.
import 'cardinal.just'

default:
    @just --list

# Format, lint and test the project, then check the specs (when the feature `specs` is on): the `check_command` of
# `.claude/cardinal.toml`, quiet on success, since the agent reads all of its output.
check:
    @cardinal run project-check
