# 0064. Exclusivity: overlapping `mut` access is a compile error, with a runtime check only where overlap can't be decided

- **Status:** Accepted; partly superseded by [ADR 0119](0119-copy-read-only-overlaps.md) (a read-only argument that overlaps a `mut` one is now copied)
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership
- **Spec rules:** [own.excl.1–3](../spec/07-ownership.md)
- **Resolves:** OQ-032

## Options considered

- **Compile error, plus a runtime check for undecidable indices (chosen):**
  guarantees the no-aliasing assumption, and overlap bugs never pass silently.
  Con: some calls need restructuring, and a few residual checks remain.
- **Implicit copy of the read-only argument:** hidden copies, masks bugs, and
  leaves `mut a[i], mut a[j]` unresolved.
- **Runtime check always:** bugs found late, a cost everywhere, and the
  optimizer can't rely on it statically.

## Decision

1. During a call, a place passed as a `mut` argument must not overlap any
   other argument of the same call.
2. If overlap is statically certain (e.g. `f(mut x, x)`), it is a
   compile-time error.
3. If overlap depends on run-time values (e.g. `swap(mut a[i], mut a[j])`),
   the compiler inserts a check that traps on overlap. Each remaining check
   is listed in the performance report.

## Consequences

- The optimizer can rely on `mut` arguments never aliasing.
- The standard library offers non-overlapping forms (`a.swap(i, j)`) where
  they avoid checks.

## Rejected alternatives

- **Implicit copy:** hidden cost, and hides bugs.
- **Runtime-only:** weaker guarantees.
