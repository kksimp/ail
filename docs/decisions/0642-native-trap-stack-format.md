# 0642. The native stack lines name the function and the call position, and leave out values

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Native runtime diagnostics
- **Spec rules:** `trace.frame.1`, `trace.snap.3`

## Context

The interpreter's frame line is `  at LINE:COL in NAME(PARAM = VALUE, ...)`,
followed by the frame's locals (trace.snap.2). Native code keeps no names or
values for its variables.

## Options considered

### Option A — `  at [FILE:]LINE:COL in NAME(..)`, the interpreter's line without values (chosen)
`NAME()` for a function without parameters. `FILE:` only outside the first
file (as in trace.run.2). The report keeps the interpreter's limit of ten
lines (trace.snap.3): the message, two excerpt lines, then up to seven
frames; when there are more than seven, six frames and
`  … N more frames`, `N` being the rest (the interpreter's line adds a
`--verbose` hint that has no native counterpart).
- **Pros:** the same positions and names in the same order, so an agent
  reads native and interpreted traps alike; the native conformance test
  strips the interpreter's values and compares the rest line for line.
- **Cons:** no argument values. To see them, run `ail run`.

### Option B — keep a table of parameter names and print their values
- **Pros:** closer to the interpreter.
- **Cons:** values need type information the runtime doesn't have and spill
  slots the allocator may reuse. Rejected.

## Decision

Option A. AI understanding: a fixed, short line.

## Consequences

- Only the trapping function's line is built at compile time (part of the
  excerpt text, `d:tK`); the callers' lines are printed by the runtime.
- Frames merged by the compiler show as described in ADR 0643.

## Rejected alternatives

Option B above, and printing every frame (a deep recursion would print
thousands of near-identical lines; the interpreter stops at ten).
