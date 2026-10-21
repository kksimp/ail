# 0628. `std.log`: leveled messages on standard error

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** std.log.1-2 ([17-std](../spec/17-std.md))

## Context

Python's `logging` is in most agent scripts: `logging.info(...)` with a level
set once. AIL programs used `io.eprint` with ad hoc prefixes and no way to
silence debug output.

## Decision

`std.log`, in AIL: `debug`, `info`, `warn` and `err` write `LEVEL message` to
standard error; the environment variable `LOG_LEVEL` (default `info`) picks
the lowest level shown (std.log.1-2). `err` rather than `error`, which is a
keyword. No timestamps, so output is deterministic; a program that wants them
prints `date.now().iso()` itself.

## Rejected alternatives

- **Global `log.set_level(...)` state.** AIL has no mutable globals; an
  environment variable also lets the person running the tool choose.
- **Structured fields (`log.info(msg, {k: v})`).** Not measured as a common
  need; add when the corpus shows it.
- **Writing to standard output.** It would mix with the program's results.
