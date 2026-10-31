# Changelog

The notable changes of each version, newest first ([Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
[Semantic Versioning](https://semver.org/)). Each section is written by `just release <version>` from the commits,
then reread before the release is committed.

## [1.2.0] - 2026-10-31

### Milestones

- Advisor

### Added

- count the advisor calls and tokens of each session
- show the advisor columns in the efficiency table
- give a verdict on the expected advisor effects
- let the close of an issue commit without a review
- name when the issue workflow consults the advisor
- show the cache read share in the efficiency table
- sum the tokens spent on each closed issue
- report the tokens per closed issue
- leave out the issues whose tokens are unknown
- require enough issues for a per-issue verdict
- narrow the delegated exploration to wide searches
- require enough issues and reviews for a ratio verdict
- flag the skills a session should have loaded
- flag the tool results that waste tokens
- weigh the bytes of the waste rules per revision
- give the bytes per session of a waste rule a verdict
- keep the waste rules out of the hook candidates
- group the report by the setup of the sessions
- split the delivered quality by setup
- compare the model setups in the harness evaluation

### Fixed

- confound an advisor verdict when the advisor set changed
- refuse a close commit that holds more than rewrites
- measure the main context peak of an advised message
- take the iteration peak only on an advised message
- settle the advisor moments the high review found loose
- cover the verdict on the cache read share
- count a resumed message once in the tokens of an issue
- shorten the delegated search and tell it from a git log
- read the skill routing triggers more strictly
- accept the waste bytes metric for a waste rule only
- read a model advised by itself as a solo setup
- settle the setup comparison the high review found loose

## [1.1.0] - 2026-10-30

### Added

- end close-issue on the next issue to work on
- end the close report on a link to the next issue

## [1.0.0] - 2026-10-30

### Milestones

- Team use

### Breaking

- rank each issue in its own file

### Added

- read the user language from a personal file
- measure the user language from the personal file
- carry the personal configuration into issue worktrees
- order the issues by the rank of each issue file
- convert the issue queue into the ranks of the files

## [0.2.0] - 2026-10-30

### Milestones

- Plan and release

First numbered release.
