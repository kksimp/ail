# Reflection: type information for compile-time code only

> **Non-normative design note,** 2026-10-20 (lane 37 of the
> [AI-first plan](ai-first-plan.md)). Topic 16. Decided by
> [ADR 0229](../decisions/0229-compile-time-reflection-only.md); normative
> rules `refl.*` in [spec chapter 16](../spec/16-reflection.md).

## 1. Who needs to know about types

| Need | Served by |
|---|---|
| An agent asking "what fields does `Version` have?" | `ail api` (the HIR, ADR 0223). Never the running program. |
| A derive generator writing an impl per field (`ToJson`, `Arbitrary`) | **`TypeInfo`**, a plain value the compiler passes to the generator (this note). |
| Dynamic dispatch on `any Trait` | The trait's method table, built at compile time (ADR 0079). No type is inspected at run time. |
| Printing a value (`"{x}"`, error reports) | `ToStr`, automatic or derived at compile time (ADR 0126). |
| Serialisation, deep equality, hashing | Automatic implementations and derives, generated at compile time. |

Every row is met at compile time. Run-time reflection (Java's
`getClass().getFields()`, Go's `reflect`, C#'s attributes at run time) would
add a second, unchecked way to reach fields, defeat dead-code elimination and
layout freedom (ADR 0013: the compiler chooses field order), and make a
program's behaviour depend on names that a rename tool can't see. AIL has
none.

## 2. `TypeInfo`

A prelude struct describing one user type, as declared:

```ail skip=bootstrap
## A type as compile-time code sees it (refl.info.1).
struct TypeInfo
    name str                    # "Version"; "Box" for a generic type
    id str                      # its `ail api` id: "type:Version"
    module str                  # the declaring module's path; "" at the root
    kind TypeKind
    error bool                  # declared with `error`
    params List[str]            # generic parameter names, in order
    fields List[FieldInfo]      # a struct's fields, in declaration order
    variants List[VariantInfo]  # an enum's variants, in declaration order

enum TypeKind
    Struct
    Enum

struct FieldInfo
    name str
    type_text str               # as written in the declaration: "List[int]"
    public bool

struct VariantInfo
    name str
    fields List[FieldInfo]
```

- **Declaration order, not layout order.** Field order in memory is the
  compiler's (ADR 0013); `TypeInfo` describes the source, which is what a
  generator writes code against.
- **Types as text.** A field's type is the text of its declaration. A
  generator emits it back into code (`self.{f.name}.to_json()`), and the
  checker resolves it at the derive site, so the generator never needs a
  structured type model. A structured form can be added as an optional field
  later without breaking generators.
- **Private fields are included.** The generated `impl` lives in the type's
  module, which may read them (mod.vis.3). `public` lets a generator choose
  to skip them.
- **Generic types:** `params` lists the parameters; the generated impl is
  checked per `gen.method.1`, with the parameters in scope. A derive for
  `Box` that needs `T ToJson` writes `impl ToJson for Box[T ToJson]`
  (`gen.method.2`); the generator sees the same `TypeInfo`.

## 3. How compile-time code obtains it

Only one way: **the compiler passes it to a derive generator** (ADR 0227).
There is no expression that produces the `TypeInfo` of a named type, at run
time or at compile time. A program may build `TypeInfo` values by hand (a
generator's tests do: `TypeInfo(name: "P", …)`), but that is plain data with
no link to any real type.

This is the smallest surface that serves the derive use case. A
`type_info[T]()` intrinsic usable in constants (for compile-time assertions
such as "every variant has a handler") is left open as
[OQ-092](../open-questions.md).

## 4. Run time

There is no run-time reflection (`refl.run.1`). Specifically:

- no operation yields a value's type, type name or field list at run time;
- `any Trait` can't be converted back to its concrete type
  ([OQ-094](../open-questions.md) records the downcast question);
- the only run-time type data is what `any Trait` dispatch needs: a method
  table per implementing type, plus `Eq` and `ToStr` of the boxed value.

## 5. Rejected alternatives

- **Run-time reflection** (Go `reflect`, Java, C#): unchecked access paths,
  hidden dependencies on names, larger binaries, and layout constraints.
  Every current use is served at compile time (§1).
- **Zig-style `@typeInfo(T)` in any compile-time code, with types as
  values:** needs compile-time type parameters (rejected in
  [compile-time-execution.md](compile-time-execution.md) §8), and lets any
  function's behaviour depend on the shape of its argument types, which
  defeats local reasoning.
- **`fields of T` syntax** (a language-level iteration over fields, as the
  plan's brief sketched): new syntax for a construct only generators use.
  A `List[FieldInfo]` is iterated with the `for` loop agents already write.
- **A structured type model in `FieldInfo`** (an enum of type shapes): more
  for every generator to handle, while text round-trips exactly through the
  checker. Can be added later as an optional field.
- **Exposing layout (offsets, sizes):** layout is the compiler's choice
  (ADR 0013) and is reported, not promised. FFI layout (`@layout(c)`) has
  its own rules.
