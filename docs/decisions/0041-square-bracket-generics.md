# 0041. Generic type arguments use `[ ]`; indexing also uses `[ ]`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 9 — Generics
- **Spec rules:** [gram.generic.1](../spec/03-grammar.md)
- **Evidence:** [syntax-measurements-r1.md](../design/syntax-measurements-r1.md)

## Context

The brief wrote `Result[T, E]`. Angle brackets are genuinely ambiguous with
the comparison operators (Rust needs `::<>`, and C++ had problems with `>>`
for years). Measured: `[]` costs up to one token more than `<>` in some
tokenizers.

## Options considered

### Option A — `[ ]` for both generics and indexing (chosen)
- **Pros:** No `<`/`>` ambiguity. Matches the brief, and reads cleanly.
- **Cons:** Up to +1 token per generic type in some tokenizers. A few rare
  expressions such as `f[T](x)` need name information to parse (Go handles
  this the same way). Type inference makes explicit type arguments uncommon.

### Option B — `< >` for generics
- **Pros:** Familiar, and up to 1 token fewer.
- **Cons:** Real parsing ambiguity, and harder to read reliably.

### Option C — `[ ]` for generics, `( )` for indexing
- **Pros:** No ambiguity at all.
- **Cons:** Indexing looks like a function call, and `a(i) = x` reads oddly.

## Decision

Generic type parameters and arguments are written in `[ ]`. Indexing also uses
`[ ]`. The two are told apart by whether the bracketed thing applies to a
type or generic function (types) or to a value (indexing).

## Consequences

- The parser either has to resolve names, or produce an ambiguous node that
  the resolver settles. That's a grammar and AST design item.
- Explicit type arguments on calls should be rare thanks to inference.
  Diagnostics should suggest them only when inference fails.

## Rejected alternatives

- **`< >`:** parsing ambiguity is worse than a one-token cost.
- **`( )` indexing:** unfamiliar and awkward for assignment.
