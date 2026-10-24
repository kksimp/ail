# 0886. Native `--values` build mode

- Status: Accepted
- Date: 2026-10-24
- Decided by: Kaleb (option A on the decision card), design by AI (delegated, ADR 0076)

## Context

The interpreter's trap report shows each frame with its arguments and locals
(`at 6:19 in down(n = 0)`, `main(); m = {1: "a"}`) and a contract trap's
`values:` line. A native executable prints `NAME(..)` (ADR 0641): keeping values
needs the frames to hold them, which costs time and memory on every call.

## Decision

Values are opt-in: `ail run --values FILE` builds with the mode `values`
(`AIL_INSTRUMENT=values`, part of the build cache key). Without it nothing
changes and nothing is paid for (ADR 0641 stays the default).

In that build each function of the program's own files:

- gets a spill object when it starts (`rt.vals_enter`), registered in a table
  under its frame pointer (global slot 61, direct-mapped, 65536 entries, scanned
  by the collector) and dropped before each return (`rt.vals_leave`);
- at the start of each statement stores a describer (`vd.N`, generated per
  statement from the names in scope and their types) and the current values of
  those names (at most 32; types the runtime can print).

A trap prints the innermost registered frame's description in place of the `(..)`
in the report's last line, and each caller's from its spill, found by the caller's
frame pointer. A contract trap prints its `values:` line. The ellipsis line names
`--verbose` as the interpreter's does.

Limits: single-threaded frames (a task's frames are not registered separately in
any special way); a trap in an unowned (std) function shows the nearest owned
caller's values.

## Rejected alternatives

- Always on: a spill store per statement for every program breaks the zero-cost
  frames of ADR 0641.
- A debug-info table for the Mac and Linux debuggers: needs per-target tables and
  a reader in the runtime for the same output, and values still have to be
  spilled to be readable at all.
- Values only for arguments: the goldens show locals (`main(); scores = [3, 4]`).
