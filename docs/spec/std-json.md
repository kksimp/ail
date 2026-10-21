# Standard library: `std.json`

Chapter key: `json`

> **Status:** Drafted. Until the module mechanism supports `use std.NAME`
> (lane 4), these items are prelude names (ADR 0161, ADR 0162). They then move
> to `use std.json` and are reached as `json.parse`, and so on.

## Types

**json.type.1** — `enum Json` has the variants `Null`, `Bool(b bool)`,
`Num(n float)`, `Str(s str)`, `Arr(items List[Json])` and
`Obj(members List[JsonMember])`. `struct JsonMember` has the fields
`key str` and `value Json`. An object keeps its members in document order,
and duplicate keys are kept. *(ADR 0161)*

```ail rules=json.type.1
fn main()
    print(Json.Arr([Json.Bool(true), Json.Str("x")]).to_text())
```

**json.err.1** — `error struct JsonError` has `at int`, the 0-based character
offset of the problem, and `message str`. *(ADR 0161)*

```ail rules=json.err.1
fn main()
    _ := parse_json("[1,]") else err
        print(err.at)
        return
```

## Parsing and printing

**json.parse.1** — `parse_json(text str) Json ! JsonError` parses exactly one
RFC 8259 document with optional surrounding whitespace. It fails on trailing
data, trailing commas, leading zeros, unescaped control characters,
unterminated strings, invalid escapes, and unpaired surrogate escapes. A
`😀` pair decodes to one character. Every number is a `float`.
All errors report the first problem found. *(ADR 0161, ADR 0162)*

```ail rules=json.parse.1
fn main()
    _ := parse_json("[1,]") else err
        print(err.message)
        return
```

**json.print.1** — `Json.to_text(self) str` gives compact text: no spaces,
members in document order, `"`, `\`, `\n`, `\r`, `\t`, `\b`, `\f` escaped by
name, other control characters as `\u00XX`, everything else as itself. A
number prints as the `float` does (`1.0`, `2.5`). The same value always gives
the same text. *(ADR 0161)*

```ail rules=json.print.1
fn main()
    v := parse_json("\{\"a\": [1, true]}") else return
    print(v.to_text())
```

**json.parse.2** — A number whose value doesn't fit a finite `float`
(`1e999`) fails with the message `number out of range` at its first
character, so a `Json.Num` read from text is always finite. *(ADR 0464)*

```ail rules=json.parse.2
fn main()
    _ := parse_json("1e999") else err
        print(err.message)
        return
```

**json.print.2** — `to_text` prints a `Num` that is infinite or NaN (only a
program can build one) as `null`, so the text it gives always parses again
and `parse_json(j.to_text())?.to_text() == j.to_text()`. *(ADR 0464)*

```ail rules=json.print.2
fn main()
    print(Json.Num(1.0 / 0.0).to_text())
```

## Querying

**json.query.1** — `Json.get(self, key str) Json?` is the first member named
`key`, and `none` for a missing key or a value that isn't an object.
`Json.at(self, i int) Json?` is element `i`, and `none` when out of range or
not an array. `as_str`, `as_num`, `as_bool`, `as_arr` and `as_obj` give the
payload of the matching variant, and `none` for any other. They chain with
`?` in a function that returns an optional, or before an `else`
(`err.prop.4`). *(ADR 0161)*

**json.query.2** — `j.str(key)`, `j.num(key)`, `j.bool(key)` and
`j.arr(key)` are `j.get(key)?.as_str()` and so on: member `key` when it holds
that variant, and `none` otherwise. *(ADR 0254)*

**json.query.3** — `j.set(key str, value Json)` (`mut self`) replaces the
value of the first member named `key`, or adds the member at the end. It traps
(`err.model.2`) unless `j` is an object. *(ADR 0254)*

```ail
fn main()
    mut doc := parse_json("\{\"id\": \"t1\", \"hours\": 2}") else return
    print(doc.str("id"))       # t1
    print(doc.str("hours"))    # none
    doc.set("done", Json.Bool(true))
    print(doc.to_text())       # {"id":"t1","hours":2.0,"done":true}
```

**json.query.4** — `j.as_int()` is the number when it has no fraction, and
`j.int(key)` is `j.get(key)?.as_int()`. `j.kind()` names the variant in
JSON's words (`null`, `bool`, `number`, `string`, `array`, `object`).
`j.need(key) Json ! JsonError` is member `key`, failing with the message
`missing KEY`; `need_str`, `need_num`, `need_int`, `need_bool` and
`need_arr` and `need_obj` give the payload, failing with `expected a
string, found KIND` and so on. These failures have `at` -1: the value isn't
tied to a place in a text. `j.non_null()` is `none` for `null`, else `j`;
`j.opt(key)` is member `key` unless it is missing or `null`;
`j.variant_name()` is a string, or the string member `kind` of an object.
*(ADR 0308, ADR 0343)*

```ail rules=json.query.4
fn main()
    v := parse_json("\{\"n\": 7}") else return
    print(v.int("n") else 0)
```

## Derives

**json.derive.1** — `trait ToJson` has `fn to_json(self) Json`, with the
generator `derive_to_json` (`ct.derive.1`). `str`, `int`, `float`, `bool`,
`Json` and `List[T ToJson]` implement it. A derived struct gives an object of
its fields in declaration order, a `T?` field that is `none` giving `null`; a
derived enum gives a variant without fields as its name, and one with fields
as an object `{"kind": "Name", fields...}`. Fields of type `Map[str, T]`,
`List[T]` and `T?` convert element by element when `T` has no `to_json`
(`List[int?]`, `Map[str, List[U]]`). A generic type derives it through
bounds on the `impl`: `impl ToJson for Pair[A ToJson, B ToJson]`.
*(ADR 0308, ADR 0343)*

**json.derive.2** — `trait FromJson` has the associated function
`fn from_json(j Json) Self ! JsonError` (`trait.assoc.1`); its generator
`derive_from_json` fills an empty `impl` with it, and `str`, `int`,
`float`, `bool`, `Json` and `List[T FromJson]` implement it. It is called as
`T.from_json(j)` (`trait.assoc.4`), in generic code too (`trait.assoc.2`). A
struct reads each field from the member of its name with `need` and the
`need_*` of its type, so a missing or mistyped member fails with its
message; a field of another type `U` calls `U.from_json`, which `U` derives
or implements too. A field may nest `List[T]`, `Map[str, T]` (read from an
object) and `T?` (missing or `null` gives `none`) to any depth. An enum
reads a variant's name, or, when a variant has fields, an object whose
member `kind` names the variant and whose other members are its fields; a
variant field named `kind` is a `ct.derive.4` error. A generic type derives
it through bounds on the `impl`, `impl FromJson for Pair[A FromJson, B
FromJson]`; without them the generated `A.from_json` is a `trait.assoc.2`
error. A map whose key isn't `str` and any other type (`Set[T]`, tuples) are
a `ct.derive.4` error naming the type. *(ADR 0308, ADR 0343, ADR 0350)*

**json.derive.3** — A failure of a derived `from_json` names its path from
the document root: `missing items[2].kind` for a missing member, and
`items[2].kind: expected a string, found number` or ``shape: unknown Shape
`Box` `` for the others. `e.under(part)`, with `part` a member name or
`[i]`, puts a `JsonError` under one more step of its path. *(ADR 0343)*

```ail
struct Item
    id str
    hours float
    tags List[str]
    note str?

impl ToJson for Item
impl FromJson for Item

fn main()
    t := Item(id: "t1", hours: 1.5, tags: ["ops"], note: none)
    text := t.to_json().to_text()
    print(text)    # {"id":"t1","hours":1.5,"tags":["ops"],"note":null}
    doc := parse_json(text) else return
    back := Item.from_json(doc) else return
    print(back == t)    # true
    bad := parse_json("\{\"id\": 7}") else return
    wrong := Item.from_json(bad) else err
        print(err.message)    # id: expected a string, found number
        return
    print(wrong)
```

## Availability

**json.prelude.1** — The items above are read only when the program mentions
`Json`, `JsonMember`, `JsonError`, `parse_json`, `ToJson` or `FromJson`. A
program that declares one of those names itself gets its own and none of the
module's. *(ADR 0162, ADR 0308)*

```ail rules=json.prelude.1
fn main()
    print(Json.Null.to_text())
```
