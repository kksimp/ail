# 0046. Error sets are inferred inside a package and explicit at package APIs; spelled `T ! E` and `!T`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 8 — Error handling; 4 — Type system
- **Spec rules:** [err.type.1–4](../spec/08-errors.md)
- **Evidence:** cl100k tokens for a signature's return type: `Result[User, DbError]` 13, `User ! DbError` 11, `!User` 9 (other tokenizers proportional)

## Context

Explicit error types in every signature lead to conversion boilerplate when
errors from different sources mix (Rust's `From` impls, wrapper enums).
Fully dynamic error types lose precision. The designer chose a hybrid.

## Options considered

### Option A — Inferred sets inside a package, explicit at the public API (chosen)
- **Pros:** No conversion boilerplate. Callers and AI can query the exact
  error list. Handling can be exhaustive. Fewest tokens.
- **Cons:** Inside a package, changing a helper changes its callers' inferred
  error sets. This is deliberately allowed, because the API boundary stays
  stable. It's a novel combination that needs careful design.

### Option B — Always explicit `T ! E`
- **Pros:** Everything is visible in every signature.
- **Cons:** Conversion boilerplate and more tokens.

### Option C — `Result[T, E]` spelled out
- **Pros:** No new syntax.
- **Cons:** The most tokens, and the same boilerplate as B.

## Decision

1. `T ! E` is the type of a function result that is either a `T` or an error `E`.
2. `!T` in a return position means "`T`, or an error set **inferred** from
   the function body": the union of every error the body can propagate or
   return.
3. Functions visible outside their package **must** state their error type
   explicitly (`T ! E`). Inference is allowed only for functions that are
   not part of a package's public API.
4. An inferred error set is a closed, exhaustively matchable union of error
   types.

## Consequences

- Open design items (OQ-030): what types may be errors (any type or marked
  error types), how a union of errors is written explicitly (the
  `IoError | ParseError` in examples is illustrative), how inferred sets
  convert to an explicit public error type, and how recursion and generic
  functions are handled during inference.
- The compiler and AI API can show the inferred set for any function, e.g.
  as an editor hint: `!Config` → `Config ! (IoError | ParseError)`.
- Inferred sets must be stable and deterministic for incremental compilation
  (ADR 0015).
- "Package" and "public" depend on the modules topic (visibility syntax such
  as `pub` is not yet decided).

## Rejected alternatives

- **Always explicit:** boilerplate.
- **`Result[T, E]`:** the most tokens, and the same boilerplate.
