# 16. Reflection

Chapter key: `refl`

> **Status:** Drafted (skeleton). Specified: the type information derive
> generators receive, and the absence of run-time reflection. The bootstrap
> implements `refl.info.1–3` (ADR 0306). Design note:
> [reflection.md](../design/reflection.md).

## Type information

**refl.info.1** — The prelude declares `TypeInfo`, `TypeKind`, `FieldInfo`
and `VariantInfo`. A `TypeInfo` describes one struct or enum as declared:
`name`, `id` (its `ail api` id, `api.id.1`), `module` (the declaring
module's path, empty at the package root), `kind` (`TypeKind.Struct` or
`TypeKind.Enum`), `is_error` (whether it is an error type; `error` is a
keyword, ADR 0306), `params` (generic
parameter names in order), `fields` (a struct's fields) and `variants` (an
enum's variants). A `FieldInfo` has `name`, `type_text` and `public`; a
`VariantInfo` has `name` and `fields`. Fields and variants are in
declaration order, not layout order (ADR 0013). The prelude also declares
`error struct DeriveError` with `message str`. They are read when the program
writes `@derive`, names one of them, or uses `std.json`. *(ADR 0229,
ADR 0306)*

```ail fragment
struct TypeInfo
    name str
    id str
    module str
    kind TypeKind
    is_error bool
    params List[str]
    fields List[FieldInfo]
    variants List[VariantInfo]

enum TypeKind
    Struct
    Enum

struct FieldInfo
    name str
    type_text str
    public bool

struct VariantInfo
    name str
    fields List[FieldInfo]
```

**refl.info.2** — The only `TypeInfo` that describes a declared type is the
one the compiler passes to a derive generator (`ct.derive.2`). No
expression yields the `TypeInfo` of a named type. A program MAY construct
`TypeInfo` values itself; they are plain data. *(ADR 0229)*

**refl.info.3** — `type_text` is the field's type as written in its
declaration, with the declaration's names, so generated code that mentions
it resolves at the deriving `impl` (`ct.gen.2`). *(ADR 0229)*

## Run time

**refl.run.1** — [definitional] There is no run-time reflection. No operation yields a
value's type, type name, fields or variants at run time, and a value of
`any Trait` can't be converted to its concrete type. *(ADR 0229)*
