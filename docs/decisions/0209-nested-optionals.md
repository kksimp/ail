# 0209. Nested optionals are distinct types, `none` is the outermost, and `T?` does not convert to `T??` (resolves OQ-044) (*delegated*)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Types; optionals
- **Spec rules:** `type.opt.6–8`

## Context

`Map[K, V?].get(k)` has type `V??`, and `none` could mean "no key" or "key
with no value". Flattening loses that information; forbidding `V?` values
in maps is restrictive (OQ-044).

## Options considered

### Option A — distinct nested types, `none` is the outer level (chosen)
- **Pros:** keeps the information; `m.get(k) else none` flattens when wanted;
  `else`, `?` and `if v := x` already peel exactly one level, so no new syntax.
- **Cons:** a value that is present but empty needs a representation.

### Option B — optionals flatten
- **Pros:** simplest type system.
- **Cons:** "missing" and "present but empty" become indistinguishable.

### Option C — `Map` values can't be optional
- **Pros:** the problem cannot arise.
- **Cons:** forbids reasonable programs and still leaves `List[T?].last()`.

## Decision

`T??` is written as such and is distinct from `T?` (`type.opt.6`). `none`
is the absent value of the outermost level, and `else`, `?` and `if v := x`
peel one level (`type.opt.7`). A `T` converts to `T??` (through `T?`,
`type.opt.5`) but a `T?` does **not**: its `none` would be ambiguous, so the
checker reports the mismatch with that help (`type.opt.8`). `Map.get_or` is
not added: `m.get(k) else d` is already as short and also works for `V?`.

## Rationale

AI understanding decided: no information is lost and no new operator is
needed. At run time a present optional is its content, except that a present
absent value is `Nested(1)`, a present one of those `Nested(2)`, and so on,
so ordinary `T?` values keep their one-word representation and speed.

## Consequences

- Printing shows a present-but-empty value as `some(none)`.
- Nested optionals arise from collection methods; there is no constructor
  yet, by design: add `some(x)` if a real program needs it.
- `==` compares levels: `m.get("b") == m.get("c")` is `false` when `"b"`
  maps to `none` and `"c"` is absent.

## Rejected alternatives

- Option B and C as above.
- `Map.get_or`: no measured saving over `else`.
