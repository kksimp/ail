# 0193. A `ref` cell traps when used through another copy during a `mut` access

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 7 — Ownership
- **Spec rules:** [own.ref.7](../spec/07-ownership.md#shared-references)

## Context

ADR 0064 makes `mut` arguments exclusive, checked statically when certain
and at run time otherwise. A `ref` breaks static reasoning: two copies of a
reference reach the same cell. `c.steal(d)`, where `fn steal(mut self, other
ref Counter)` and `d` is a copy of `c`, reads `d` while `self` is being
changed.

## Options considered

- **Trap on use through another copy (chosen):** while a `mut self` method,
  `mut` argument or `for mut` loop has a cell's contents, any use of that
  cell traps (own.ref.7). Like Swift's dynamic exclusivity for class
  properties. Cost: one check per access through a `ref`.
- **Let it happen:** the method sees a stale or half-changed value, and the
  write-back silently discards the other change. A wrong answer, not a trap.
- **Copy the contents for each `mut` access:** no trap, but lost updates and
  a deep copy per call.

## Decision

A use of a cell through any copy of a `ref` while a `mut` access to its
contents is in progress traps with `own.ref.7`.

## Rationale

AI understanding decided (criterion 2): a precise trap at the conflicting
use beats a silent lost update; it extends ADR 0064 (exclusivity of `mut`
access) to the one place static checking can't reach.

## Consequences

- The bootstrap moves a cell's contents out for the `mut` access and traps
  on finding the cell empty, so programs without `ref` pay nothing.

## Rejected alternatives

- **Let it happen:** silent wrong results.
- **Copy per access:** lost updates and copies.
