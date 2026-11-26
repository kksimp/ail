# 0044. Names may not shadow other visible names

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 4 — Type system (name resolution)
- **Spec rules:** [expr.bind.4](../spec/05-expressions.md)
- **Resolves:** the shadowing question left open in [ADR 0039](0039-declaration-syntax.md)
- **Amended by:** [ADR 1180](1180-literal-parameters-may-hide-a-local.md) (a function literal's parameters and a comprehension's variables may hide a local)

## Context

Shadowing means that one name refers to different things in different places.
That's a common source of misreading for both humans and AI
([ai-author-wishlist.md 1.2](../design/ai-author-wishlist.md)).

## Options considered

### Option A — No shadowing (chosen)
- **Pros:** One name means one thing wherever it appears. This eliminates
  the wrong-`x` class of bugs, and it's the simplest rule.
- **Cons:** Transformation chains need distinct names (`raw`, `parsed`,
  `valid`), which is occasionally verbose.

### Option B — Allow in nested scopes
- **Pros:** Short names in small blocks.
- **Cons:** Readers must work out which declaration is meant, and moving code
  between scopes can silently change meaning.

### Option C — Allow everywhere (Rust-style)
- **Pros:** Neat transformation chains.
- **Cons:** Contradicts ADR 0039, gives the most room for misreading, and
  weakens immutable bindings.

## Decision

Declaring a local name (variable, parameter, or loop or pattern binding) that
is already visible from an enclosing local scope of the same function is a
compile-time error.

## Consequences

- The diagnostic names both declarations and offers rename fixes.
- **Open detail (OQ-028):** how local names interact with module-level,
  imported and built-in names, e.g. whether a local `len` may coexist with
  a library function `len`. To be decided with the modules topic.
- Closures and nested functions follow the same rule for names they can see.

## Rejected alternatives

- **Nested-scope shadowing:** silent meaning changes when code moves.
- **Rust-style shadowing:** conflicts with ADR 0039 and ADR 0038.
