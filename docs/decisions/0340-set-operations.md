# 0340. `union`, `intersect` and `difference` on sets and lists

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Collections, token efficiency
- **Spec rules:** `type.coll.16`

## Context

Lane 83 ([program-measurements-r8.md](../design/program-measurements-r8.md))
found `snapdiff` 4.6% over its Python twin; 9.6 of the extra tokens are
`{n for keys in [old.keys(), new.keys()] for n in keys}`, where Python writes
`old.keys() | new.keys()`. `Set` had only `is_subset`.

## Options considered

### Option A — methods `union`, `intersect`, `difference` (chosen)
- **Pros:** measured −9.6 tokens on `snapdiff` (mean of five tokenizers); no
  new operator; reads the same on sets and on `keys()`.
- **Cons:** three more method names to learn.

### Option B — `|`, `&`, `-` on sets
- **Pros:** similar saving.
- **Cons:** a new overload of three operators (`-` and `|` already mean number
  and bit operations), and `keys()` is a list, so it needs more rules.

### Option C — leave it
- **Cons:** the measured gap stays.

## Decision

`s.union(t)`, `s.intersect(t)` and `s.difference(t)` return a new `Set[T]`
(`T` MUST be `Hash`). A `List[T]` has the same three methods taking a
`List[T]`, so `m.keys().union(n.keys())` works. The result keeps the
receiver's order, then the argument's new elements (union only).

## Rationale

Tokens decide (ADR 0076): `snapdiff` falls from 324.2 to 314.6 tokens. The
corpus of ten agent programs is now 5.7% under Python (3,800.0 against
3,814.4 before, where Python is 4,031.8).

## Consequences

- Resolves OQ-110. `snapdiff` and its corpus copy use `union`.
- A set operation on a list deduplicates: `[1, 1].union([])` is a set of one.

## Rejected alternatives

- Operators on sets: more rules for the same saving (Option B).
- Leaving it: the gap is measured and the fix is small (Option C).
