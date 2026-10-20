# 0252. A `?` on an optional inside an `else` value goes to that `else`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Errors and absence; token efficiency
- **Spec rules:** err.prop.2 (amended), err.prop.4, err.else.6 (amended)

## Context

`j.get(k)?.as_str()` was legal only in a function that returns an optional
(err.prop.2, ADR 0051), so `tasks` and `build` needed helper functions
(`str_field`, `num_field`, `to_finding`) where Python writes the chain
inline. Most chains end in a fallback anyway: `... else default`.

## Options considered

### Option A — the nearest enclosing `else` catches the `?` (chosen)
- **Pros:** chains work in any function when a fallback is given; no new
  operator; the reading is TypeScript's `a?.b ?? d`. Without it (and without
  ADR 0254's getters) the corpus is 85.2 tokens (4.7%) heavier.
- **Cons:** a `?` now means "leave the function" or "go to the `else`"
  depending on where it is; inside an optional-returning function an `else`
  now catches a `?` that used to return `none` (no program in the repository
  relied on that).

### Option B — a separate `?.` chaining operator
- **Cons:** changes what `x?.f` means in every existing optional function;
  two operators that look alike.

### Option C — keep helper functions
- **Cons:** the tokens above, and a helper per field type.

## Decision

A `?` on an optional inside the value `e` of `e else ...` (binding,
expression or `for` header form) goes to the nearest such `else` when it
finds `none`; `e` then counts as absent. Result `?` is unchanged. `else err`
on a result whose value also contains a caught `?` is an error. A `for`
header accepts `e else d` (`for x in j.arr("k") else []`, −4.8 tokens).

## Rationale

The token gain overlaps with ADR 0254 (typed getters remove most of the
`?`s in the corpus), so on the corpus alone it is a tie; **AI understanding**
decided: the chain-then-fallback form is what models write from TypeScript,
Kotlin and C#, and the error it removes ("`?` only works in a function that
returns an optional") pushed models into helper functions.

## Consequences

- The checker marks caught `?`s; the interpreter turns their early return
  into absence at the `else`.
- The err.prop.2 diagnostic now suggests the `else` form.

## Rejected alternatives

- `?.` as a new operator: changes existing code's meaning.
- Helpers: tokens.
