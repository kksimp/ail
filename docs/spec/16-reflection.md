# 16. Reflection

Chapter key: `refl`

> **Status:** Drafted (skeleton). Specified: the type information derive
> generators receive, and the absence of run-time reflection. The compiler
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
enum's variants). A `FieldInfo` has `name`, `type_text`, `public` and `doc` (the field's `##`
comment, `ct.derive.5`); a
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
    doc str

struct VariantInfo
    name str
    fields List[FieldInfo]
```

```ail rules=refl.info.1
fn main()
    t := TypeInfo(name: "Point", id: "geo.Point", module: "geo", kind: TypeKind.Struct, is_error: false, params: [], fields: [FieldInfo(name: "x", type_text: "int", public: true, doc: "")], variants: [])
    print("{t.name} {t.fields[0].name} {t.fields[0].type_text}")
```

**refl.info.2** — The only `TypeInfo` that describes a declared type is the
one the compiler passes to a derive generator (`ct.derive.2`). No
expression yields the `TypeInfo` of a named type. A program MAY construct
`TypeInfo` values itself; they are plain data. *(ADR 0229)*

```ail rules=refl.info.2
fn main()
    fake := TypeInfo(name: "Ghost", id: "Ghost", module: "", kind: TypeKind.Enum, is_error: true, params: ["T"], fields: [], variants: [VariantInfo(name: "Gone", fields: [])])
    print("{fake.name} {fake.is_error} {fake.params.len()} {fake.variants[0].name}")
```

**refl.info.3** — `type_text` is the field's type as written in its
declaration, with the declaration's names, so generated code that mentions
it resolves at the deriving `impl` (`ct.gen.2`). *(ADR 0229)*

```ail rules=refl.info.3
struct Pair
    first List[Map[str, int]]
    second int

fn main()
    f := FieldInfo(name: "first", type_text: "List[Map[str, int]]", public: false, doc: "")
    print("{f.name} has type {f.type_text}")
```

## Run time

**refl.run.1** — [definitional] There is no run-time reflection. No operation yields a
value's type, type name, fields or variants at run time, and a value of
`any Trait` can't be converted to its concrete type. *(ADR 0229, ADR 0877)*

```ail rules=refl.run.1
trait Shape
    fn area(self) float

struct Square
    side float

impl Shape for Square
    fn area(self) float
        self.side * self.side

fn describe(s any Shape) float
    s.area()

fn main()
    print(describe(Square(side: 2.0)))
```
