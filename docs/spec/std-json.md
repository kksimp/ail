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

**json.err.1** — `error struct JsonError` has `at int`, the 0-based character
offset of the problem, and `message str`. *(ADR 0161)*

## Parsing and printing

**json.parse.1** — `parse_json(text str) Json ! JsonError` parses exactly one
RFC 8259 document with optional surrounding whitespace. It fails on trailing
data, trailing commas, leading zeros, unescaped control characters,
unterminated strings, invalid escapes, and unpaired surrogate escapes. A
`😀` pair decodes to one character. Every number is a `float`.
All errors report the first problem found. *(ADR 0161, ADR 0162)*

**json.print.1** — `Json.to_text(self) str` gives compact text: no spaces,
members in document order, `"`, `\`, `\n`, `\r`, `\t`, `\b`, `\f` escaped by
name, other control characters as `\u00XX`, everything else as itself. A
number prints as the `float` does (`1.0`, `2.5`). The same value always gives
the same text. *(ADR 0161)*

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
`need_arr` give the payload, failing with `expected a string, found KIND`
and so on. These failures have `at` -1: the value isn't tied to a place in
a text. *(ADR 0308)*

## Derives

**json.derive.1** — `trait ToJson` has `fn to_json(self) Json`, with the
generator `derive_to_json` (`ct.derive.1`). `str`, `int`, `float`, `bool`,
`Json` and `List[T ToJson]` implement it. A derived struct gives an object of
its fields in declaration order, a `T?` field that is `none` giving `null`; a
derived enum gives a variant without fields as its name, and one with fields
as `{"Name": {fields}}`. *(ADR 0308)*

**json.derive.2** — `trait FromJson` has no methods; its generator
`derive_from_json` adds the associated function `T.from_json(j Json) T !
JsonError` (`ct.gen.1`). A struct reads each field from the member of its
name with `need` and the `need_*` of its type, so a missing or mistyped
member fails with its message; a field of another type `U` calls
`U.from_json`, which `U` derives too; a field `T?` of `str`, `int`, `float`,
`bool` or `Json` may be missing. An enum whose variants have no fields reads
a variant's name. Generic types, other optional fields, maps and variants
with fields are a `ct.derive.4` error naming the field. *(ADR 0308)*

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
        print(err.message)    # expected a string, found number
        return
    print(wrong)
```

## Availability

**json.prelude.1** — The items above are read only when the program mentions
`Json`, `JsonMember`, `JsonError`, `parse_json`, `ToJson` or `FromJson`. A
program that declares one of those names itself gets its own and none of the
module's. *(ADR 0162, ADR 0308)*
