# 0090. Collections: `List[T]`, `Map[K, V]`, `Set[T]`, with `[...]` and `{...}` literals

- **Status:** Accepted; amended by [ADR 0143](0143-empty-literals-typed-by-use.md) (an empty literal declaring a local is typed by its first use)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 21 — Standard library
- **Spec rules:** [type.coll.1–4](../spec/04-types.md)
- **Evidence:** `tools/tokbench/corpus/r4/` map literal 21.6 vs. `Map.of(...)` 24.4 (−11%); `List[T]` 43.0 vs. `[T]` 41.4 (identical on cl100k and o200k; the gap comes only from older tokenizers, and only in signature-heavy code)

## Deciding criterion

**Token usage** decides the literals. For the type spelling, tokens **tie**
(identical on modern tokenizers), so **AI understanding** decides:
`List[T]` can't be confused with a list literal `[x]` or a generic parameter
list `[T Bound]`.

## Decision

1. `List[T]` (growable sequence), `Map[K, V]` and `Set[T]` are standard types
   with value semantics (ADR 0060).
2. List literal: `[a, b, c]`. Map literal: `{key: value, ...}`. Set literal:
   `{a, b}`.
3. The empty literals `[]` and `{}` require a known expected type. Without
   one, it's a compile-time error suggesting `List[T]()`, `Map[K, V]()` or
   `Set[T]()`.
4. `xs[i]` indexes (traps when out of bounds, per ADR 0045), and
   `xs[a..b]` produces the sub-list as a value that shares storage.

## Consequences

- Fixed-size arrays and SIMD vectors need const generics, which are to be
  designed.
- Iteration protocol and associated types are tied to OQ-037.

## Rejected alternatives

- `Map.of(...)` for map construction: 24.4 tokens, against 21.6 for the `{key: value}` literal (about 11% more). Rejected on token usage.
- Bare `[T]` for lists: identical on modern tokenizers (41.4 against 43.0 for `List[T]`), so AI understanding decides. `List[T]` is not confused with a list literal `[x]` or a generic parameter list `[T Bound]`.
- Typing an empty literal from an annotation only, with no use-site inference: later amended by [ADR 0143](0143-empty-literals-typed-by-use.md), which lets an empty literal that declares a local take its type from first use.
