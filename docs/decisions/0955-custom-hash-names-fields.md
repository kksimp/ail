# 0955. Custom hashing: `impl Hash` names the fields that take part

- **Status:** Proposed
- **Date:** 2026-11-05
- **Decided by:** Designer (2026-11-05, card answer: option A, `impl Hash` names the fields)
- **Topic:** 10 — Traits
- **Spec rules:** [trait.hash.1-3](../spec/10-traits.md)
- **Amends:** [ADR 0133](0133-automatic-hash.md) (`Hash` stays automatic; a hand-written `impl Hash` is now allowed for the case below)
- **Resolves:** OQ-042

## Context

A hand-written `fn eq` removes `Hash` (trait.auto.5), so a type with its own equality can't be a map key or set element. Hash values must stay invisible to programs.

## Decision

1. `impl Hash for T` with one `uses` line naming fields restores `Hash` for a type whose `fn eq` is hand-written: `impl Hash for Key` then, indented, `uses id, region` (trait.hash.1). Without `impl Hash`, ADR 0133 is unchanged.
2. Only the named fields are hashed, in the order written, with the automatic rule for each field's type. Each named field MUST itself be `Hash` (trait.hash.2).
3. Consistency: equal values must hash equal, so every named field must be one that `eq` requires to be equal. The checker warns when the `eq` body never reads a named field (trait.hash.3). A hash that ignores a field `eq` compares is always safe; only extra hashed fields can miss.
4. `uses` is the only form. No `fn hash`, no `Hasher` value, so hash values stay unobservable and output cannot depend on the hash function.

## Rejected alternatives

- `fn hash(self, mut h Hasher)` (Rust style): a user-written hash can disagree with `eq`, and exposes a hasher type. Designer chose the field list.
- Keep `Hash` automatic only: custom-`eq` types stay unusable as keys.

## Consequences

The checker change goes in `compiler/check/traits.ail`; map lowering reuses the automatic hasher over the named fields. Status moves to Accepted when the implementation passes the gate.
