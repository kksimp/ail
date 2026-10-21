# 0733. Process results have `text()` and `ok()`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 234)
- **Topic:** Standard library (`proc.Output`)
- **Spec rules:** std.proc.9, std.proc.10

## Context

`proc.run` returns `proc.Output` with `status`, `out` and `err`. Agent programs repeatedly
build the combined text (`"{r.out}{r.err}"`, because a string can't be joined with `+`) and
test for success (`r.status == 0`). Round 5 measured both as repeated boilerplate.

## Decision

1. `o.text()` is standard output followed by standard error (`std.proc.10`).
2. `o.ok()` is `o.status == 0` (`std.proc.10`).
3. Both are methods called with parentheses, like the language's other methods (`s.trim()`), and
   they take no arguments. The checker rejects an argument or another name (`std.proc.9`).

They are declared in the checker's method table (`bootstrap/src/stdlib.rs`, `OUTPUT_METHODS`)
and dispatched through `StdCall::Output`, as the tensor methods are.

## Rejected alternatives

- **Fields `r.text` and `r.ok` without parentheses.** Rejected: the language has no computed
  fields, and a field and a method with one name would read the same as a `struct` field.
- **Putting the methods in `std/types.ail`.** Tried: std types' declarations there do not
  register methods, so the method table is the place that works.
- **A `text` that puts stderr first.** Rejected: the order of a command's output (stdout then
  stderr) is the one a reader expects.

## Consequences

- `fix_preview` and `test_coverage` in the round-5 corpus lose the `"{r.out}{r.err}"` pattern and
  use `r.text()`; their output is unchanged.
