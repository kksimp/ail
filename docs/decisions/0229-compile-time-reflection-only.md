# 0229. Reflection is compile-time only: generators receive a `TypeInfo` value

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 16 — Reflection
- **Spec rules:** [refl.info.1–3](../spec/16-reflection.md), [refl.run.1](../spec/16-reflection.md)
- **Design note:** [reflection.md](../design/reflection.md)

## Context

Derive generators (ADR 0227) need a type's fields and variants. Agents get
type information from `ail api`, and `any Trait` dispatch uses method
tables built at compile time. Run-time reflection would add unchecked
access paths and constrain layout (ADR 0013). **AI understanding decides**
(local reasoning; renames find every use), then speed (dead-code
elimination and layout freedom).

## Options considered

### Option A — a plain `TypeInfo` value, passed only to generators (chosen)
- **Pros:** iterated with the `for` loops agents already write; no new
  syntax; generators can be tested with hand-built values.
- **Cons:** compile-time assertions over a type's shape need a later
  intrinsic (OQ-092).

### Option B — `@typeInfo(T)` anywhere at compile time (Zig)
- **Pros:** general.
- **Cons:** needs types as values; any function's behaviour could depend on
  its argument types' shape.

### Option C — run-time reflection (Go, Java)
- **Pros:** generic serialisation without generation.
- **Cons:** unchecked, slow, hides dependencies, fixes layout.

### Option D — `fields of T` syntax
- **Pros:** reads well.
- **Cons:** new syntax for a construct only generators use.

## Decision

1. `TypeInfo`, `TypeKind`, `FieldInfo` and `VariantInfo` are prelude types
   describing a struct or enum as declared: name, id, module, kind, `error`,
   generic parameters, fields (name, type text, `public`) and variants, in
   declaration order (`refl.info.1`).
2. The compiler passes one to a derive generator; no expression yields the
   `TypeInfo` of a named type (`refl.info.2`).
3. Field types are given as their declaration text (`refl.info.3`).
4. There is no run-time reflection: no operation yields a value's type,
   type name or fields at run time, and `any Trait` can't be converted back
   to its concrete type (`refl.run.1`).

## Consequences

- **Designer, look first at:** "no downcast from `any Trait`" (OQ-094),
  which some programs will want.
- Layout stays the compiler's (ADR 0013): `TypeInfo` shows declaration
  order, not memory order.
- The bootstrap doesn't declare these types yet; they arrive with
  generators.

## Rejected alternatives

- **Types as values (B):** breaks local reasoning; rejected with Zig
  `comptime` in ADR 0225.
- **Run-time reflection (C):** every current use is met at compile time.
- **`fields of T` (D):** syntax for a rare construct.
- **Structured field types:** more for every generator to handle; can be
  added as an optional field.
