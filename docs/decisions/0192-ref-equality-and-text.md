# 0192. `ref` values compare by identity, show their contents, and writes through them are `mutates`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 7 — Ownership; 18 — Effects
- **Spec rules:** [own.ref.5–6](../spec/07-ownership.md#shared-references), [eff.infer.2](../spec/18-effects.md)

## Context

ADR 0063 and ADR 0067 define `ref T` but leave `==`, the text form, map
keys, constants and effects open.

## Options considered

### `==` on `ref T`
- **Identity (chosen):** two refs are equal when they share a cell. Cheap,
  always terminates (refs are how cycles are built), and matches what a
  `ref` is for: identity.
- **Contents:** compares the `T`s, but loops forever on a cyclic graph and
  makes `a == b` not tell whether a change to `a` shows in `b`.

### Text form
- **`ref` + contents, with `ref ...` for a cell met again (chosen):** shows
  the data, says it is shared, and terminates on cycles.
- **An address:** stable output is a goal (tests compare text), and an
  address says nothing about the data.

### Others
- `ref T` is not `Ord` or `Hash` (identity order and hashing would make
  output depend on allocation). A constant can't contain a `ref` (it would be
  one shared mutable global). `ref T?` is an optional reference.
- A function that writes through a `ref` has `mutates` (eff.infer.2): its
  signature has no `mut`, but the write is visible to the caller.

## Decision

`==` on refs is identity; the text form is `ref` and the contents, with
cycles shown as `ref ...`; refs aren't keys, aren't ordered, and aren't in
constants; writes through a ref give `mutates`.

## Rationale

AI understanding decided (criterion 2): every choice terminates and has one
meaning, and `ail effects` can answer "can this change shared state" without
reading bodies.

## Consequences

- Comparing contents is written field by field, or with a method.

## Rejected alternatives

- **Contents equality:** doesn't terminate on cycles.
- **Address text form:** unstable output.
