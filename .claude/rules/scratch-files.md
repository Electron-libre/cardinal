# Scratch files

`$TMPDIR` is a directory of this session, removed when it ends. Write scratch files (measures, logs, smoke runs,
copies of a repository, browser profiles) under it, through `mktemp` or `"$TMPDIR/<name>"`, never to a literal
`/tmp/<name>` path, which outlives the session. For an artefact the user reads after the session, create
`$TMPDIR/cardinal-keep`, or write it where the user asks.
