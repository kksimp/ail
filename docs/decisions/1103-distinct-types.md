# 1103. Distinct types: `type UserId = int`

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076); the designer approved item 15 of the AI-first review (distinct named types) for 0.1.0 on 2026-11-12.
- **Topic:** Types ([04-types.md](../spec/04-types.md), Distinct types)
- **Spec rules:** type.distinct.1-7

## Context

Programs pass many values of one base type that must never be swapped: user
IDs and order IDs, cents and counts, metres and seconds. In AIL they were all
`int` or `float`, so `charge(order_id, user_id)` checked. The workaround, a
struct of one field, is noisy (`UserId(value: 3)`, `id.value`), loses the
operators, and costs a heap object per value in native code (a struct is a
pointer, ADR 0392). Agents bring the habit from Go (`type UserId int`) and
Rust (`struct UserId(u64)`).

## Decision

`type Name = Base` at the top level declares a distinct type over a built-in
scalar: `int`, a sized number, `float`, `str`, `bool`, `char` or `bytes`.

- **Nominal.** `UserId` is never `int` and never `OrderId`; passing one for
  another is an ordinary type error.
- **In:** `UserId(x)`, one positional value of the base; a literal works there
  (`UserId(3)`). Nothing converts implicitly anywhere else.
- **Out:** `int(id)`, the base type's name, Go's spelling. It is the mirror of
  `UserId(3)`, it needs no field or method name, and it reads the same for
  every base (`str(name)`, `float(m)`). The name must be the base: `float(id)`
  on an `int`-based type is an error naming the right one.
- **Operators.** The base's `+ - * / %` take two operands of the same distinct
  type and give it back; unary `-` and `not` keep it; comparisons give `bool`.
  `id + 1` is an error whose help shows both fixes (`id + UserId(1)` or
  `int(id) + 1`).
- **Standard traits pass through.** `Eq`, `Ord`, `Hash`, `ToStr`, `ToJson`,
  `FromJson` and `Default` behave as the base's: `"{id}"` prints `7`, a map key
  and `sorted()` work, and derived JSON writes and reads the bare value.
- **No fields, no methods, no `impl`.** `id.value` and `id.abs()` are errors
  whose help shows the conversion; a method or `impl` declared on the type is
  an error. Behaviour goes in functions that take the type.
- **Native: zero cost.** Before lowering, every distinct type is erased to its
  base (`compiler/mir/lower_distinct.ail`): declarations go, written types
  become the base, `UserId(x)` and `int(x)` become `x`, and the derive's
  `UserId.from_json(j)` and `UserId.default()` become `j.need_int()` and `0`.
  A `List[UserId]` is a list of ints.

The checker sees the declaration as a struct of one field marked
`is_distinct` (`StructDecl.is_distinct`), so name resolution, modules, `pub`,
generics and error messages treat it as any declared type with no new `Ty`
variant; `compiler/check/distinct.ail` adds the rules above in a few hooks.
The base list is fixed so the erasure needs no type information.

## Options considered

### A. Distinct type over a scalar, erased in native code (chosen)
- **Pros:** catches swapped IDs and units at the call site; reads like the
  Go habit agents already have; no run-time cost; small checker change.
- **Cons:** no methods yet; the base must be a scalar.

### B. A transparent alias, `type Name = T` meaning the same type
- **Rejected:** catches nothing; a reader must look up whether two names are
  the same type, against local reasoning (ADR 1077, criterion 1). The review
  item asked for a distinct type.

### C. One-field struct sugar (`struct UserId(int)`)
- **Rejected:** keeps the heap object per value (runtime cost users can't opt
  out of) and needs a field or method name to unwrap.

### D. `.value()` to convert back
- **Rejected:** a method call on a type that otherwise has no methods; erasing
  it needs the receiver's type, which the native lowering doesn't have before
  lowering, and a struct may have its own `value()`. `int(id)` is syntactic,
  symmetric with `UserId(3)`, and is what Go writes.

### E. Base literals convert implicitly anywhere (`id == 3`, Go's untyped constants)
- **Rejected:** a bare `3` where a `UserId` is expected is the mistake the type
  exists to catch; the explicit `UserId(3)` costs two tokens.

### F. Methods and `impl`s on distinct types now
- **Deferred (OQ-144):** after erasure two distinct types over `int` would both
  put methods on `int`, so methods need name mangling by the declared type and
  dispatch on the checker's types. Free functions cover the need for 0.1.0.

## Consequences

- A program can declare `type Cents = int` and pass `Cents` through lists,
  maps, JSON and generic functions with no conversions inside them.
- Matching a distinct value against literal patterns is not supported; match
  on `int(x)`.
- `type` stays a plain word everywhere but the start of a top-level line
  followed by a name.
