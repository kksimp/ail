# 1037. No downcasting of `any Trait`

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24)
- **Resolves:** OQ-094
- **Spec rules:** `refl.run.1`
- **Topic:** 16 — Reflection

## Context

`refl.run.1` gives no way to recover the concrete type of an `any Trait` value. OQ-094 asked
whether to add one.

## Decision

There is none. An `any Trait` value offers its trait's methods and nothing else. A program
that needs to know which of several known types it holds uses an enum, whose `match` is
checked for exhaustiveness. A trait that wants it may declare its own method, such as
`fn as_circle(self) Circle?`.

## Consequences

Adding a type that implements a trait never silently skips code that branched on the
concrete types, because no such code can exist. `any Trait` keeps only its method table.

## Rejected alternatives

- **A checked `x as T?`** (returns `none` on a mismatch): cheap at run time, since the method
  table already identifies the type, but a new type is never flagged at the places that test
  for each one, and it reopens run-time type tests that `refl.run.1` rules out. It can still
  be added later without breaking code.
- **A built-in, author-written downcast trait:** the same pattern is already possible with
  an ordinary trait method, so it needs no language support.
