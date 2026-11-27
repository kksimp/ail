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

**json.err.2** — A `parse_json` failure in the shape of the text fails with
`JsonError` and `message` `unexpected character 'C'`, `unexpected end of input`,
`extra data after value`, `expected ':' after object key` or `trailing comma`. `at`
is the character where the parser stopped. *(ADR 0783)*

**json.err.3** — A failure inside a string fails with `JsonError` and `message`
`unterminated string`, `control character in string`, `invalid escape '\C'`,
`invalid unicode escape` or `unpaired surrogate escape`. `at` is the backslash or
the opening quote. *(ADR 0783)*

**json.err.4** — A malformed number fails with `JsonError` and `message` `invalid
number`; a number too large for a float fails with `number out of range`. *(ADR 0783)*

```ail rules=json.err.2
fn why(text str) str
    j := parse_json(text) else err
        return "{err.at}: {err.message}"
    return "ok {j.kind()}"

fn main()
    print(why("[1,]"))     # 2: trailing comma
    print(why("[1] 2"))    # 4: extra data after value
    print(why("@"))        # 0: unexpected character '@'
```

```ail rules=json.err.3
fn why(text str) str
    j := parse_json(text) else err
        return "{err.at}: {err.message}"
    return "ok {j.kind()}"

fn main()
    print(why("\"abc"))       # 0: unterminated string
    print(why("\"a\\qb\""))  # 2: invalid escape '\q'
```

```ail rules=json.err.4
fn why(text str) str
    j := parse_json(text) else err
        return "{err.at}: {err.message}"
    return "ok {j.kind()}"

fn main()
    print(why("-"))      # 1: invalid number
    print(why("1e999")) # 0: number out of range
```

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
number prints as the `float` does (`2.5`), except that a whole number of
magnitude below 1e15 prints without a fraction (`3`, `-12`, `0`; the sign of
a zero is dropped), so an `int` field derived through `ToJson` reads back
as an integer in other languages. The same value always gives the same text.
*(ADR 0161, ADR 0631)*

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

**json.parse.3** — Arrays and objects nest at most 512 deep in `parse_json`: the
opening bracket of a 513th level fails with `JsonError`, `message` `nesting
deeper than 512` and `at` that bracket, so text from outside can't exhaust the
stack. `parse_json_with(text str, max_depth int) Json ! JsonError` is
`parse_json` with another limit; a scalar document is at depth 0 and always
passes. *(ADR 1302)*

```ail rules=json.parse.3
fn main()
    deep := ["[".repeat(600), "]".repeat(600)].join("")
    _ := parse_json(deep) else err
        print("{err.at}: {err.message}")    # 512: nesting deeper than 512
        v := parse_json_with(deep, 600) else return
        print(v.kind())                     # array
        return
```

**json.print.2** — `to_text` prints a `Num` that is infinite or NaN (only a
program can build one) as `null`, so the text it gives always parses again
and `parse_json(j.to_text())?.to_text() == j.to_text()`. *(ADR 0464)*

```ail rules=json.print.2
fn main()
    print(Json.Num(1.0 / 0.0).to_text())
```

**json.print.3** — `Json` implements `ToStr` (`trait.std.3`) with the text of
`to_text`: `j.to_str()`, `print(j)` and `"{j}"` write JSON, and a `Json` inside
a list or another value prints as its JSON text too. *(ADR 1183)*

```ail rules=json.print.3
fn main()
    v := parse_json("\{\"a\": [1, \"x\"]}") else return
    print(v)                    # {"a":[1,"x"]}
    print(v.to_str())           # {"a":[1,"x"]}
    print("{v.get("a") else Json.Null}")   # [1,"x"]
```

## Querying

**json.query.1** — `Json.get(self, key str) Json?` is the first member named
`key`, and `none` for a missing key or a value that isn't an object.
`Json.at(self, i int) Json?` is element `i`, and `none` when out of range or
not an array. `as_str`, `as_num`, `as_bool`, `as_arr` and `as_obj` give the
payload of the matching variant, and `none` for any other. They chain with
`?` in a function that returns an optional, or before an `else`
(`err.prop.4`). *(ADR 0161)*

```ail rules=json.query.1
fn main()
    doc := parse_json("\{\"id\": \"t1\"}") else return
    id := doc.get("id") else return
    print(id.as_str() else "none")
    gone := doc.get("missing") else
        print("no member")
        return
    print(gone.as_str())
```

**json.query.2** — `j.str(key)`, `j.num(key)`, `j.bool(key)` and
`j.arr(key)` are `j.get(key)?.as_str()` and so on: member `key` when it holds
that variant, and `none` otherwise. *(ADR 0254)*

**json.query.3** — `j.set(key str, value Json)` (`mut self`) replaces the
value of the first member named `key`, or adds the member at the end. It traps
(`err.model.2`) unless `j` is an object. *(ADR 0254)*

```ail rules=json.query.2,json.query.3
fn main()
    mut doc := parse_json("\{\"id\": \"t1\", \"hours\": 2}") else return
    print(doc.str("id"))       # t1
    print(doc.str("hours"))    # none
    doc.set("done", Json.Bool(true))
    print(doc.to_text())       # {"id":"t1","hours":2,"done":true}
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

**json.query.5** — `j.text(key str, default str) str` is the text of member
`key` when it holds a string, and `default` otherwise: when the member is missing,
or holds another kind. It never fails. *(ADR 0732)*

**json.query.6** — `j.flag(key str, default bool) bool` is the same for a bool
member: `j.bool(key)` or `default`. *(ADR 0732)*

```ail rules=json.query.5,json.query.6
fn main() !
    doc := parse_json("\{\"name\": \"ada\", \"admin\": true}")?
    print(doc.text("name", "anon"))       # ada
    print(doc.text("email", "none"))      # none
    print(doc.flag("admin", false))       # true
    print(doc.flag("active", false))      # false
```

**json.pointer.1** — `j.pointer(path str) Json?` reads a JSON Pointer
(RFC 6901). `""` is `j` itself. Details: [below](#more-on-jsonpointer1). *(ADR 1232)*

```ail rules=json.pointer.1
fn main()
    doc := parse_json("\{\"a/b\": [10, 20]}") else return
    print(doc.pointer("/a~1b/1") else Json.Null)    # 20
    print(doc.pointer("/a~1b/2") else Json.Null)    # null
```

**json.merge.1** — `target.merge(patch Json) Json` applies a JSON Merge Patch
(RFC 7386) and gives the result. When `patch` is not an object it is the
result. When it is an object, `target` is taken as an object (an empty one if it
is not), and each member of `patch` in order either removes every member of
that key when its value is `null`, or replaces the member's value with
`old.merge(value)` (`old` is `null` when the key is new, so a `null` inside a
new object is dropped too), adding the member at the end when the key is new.
Arrays are replaced whole, never merged. A target's own `null` members stay. *(ADR 1232)*

```ail rules=json.merge.1
fn main()
    t := parse_json("\{\"a\": \{\"b\": 1, \"c\": 2}, \"d\": [1]}") else return
    p := parse_json("\{\"a\": \{\"c\": null, \"e\": 3}, \"d\": [2]}") else return
    print(t.merge(p))     # {"a":{"b":1,"e":3},"d":[2]}
```

**json.lines.1** — `parse_json_lines(text str) List[Json]` is each line of `text`
that parses as one JSON document, in order. A line that doesn't parse is skipped,
so a log with some broken lines still gives the good ones. *(ADR 0732)*

```ail rules=json.lines.1
fn main()
    docs := parse_json_lines("1\nnot json\n\"two\"\n")
    print(docs.len())                     # 2
```

**json.load.1** — `load_json[T FromJson](path str) T ! JsonError` reads the file at
`path` and decodes it as a `T` (`T.from_json(parse_json(text)?)?`, so a `List[U]`
of derived structs works). A file that can't be read fails with a `JsonError`
whose `message` is the path, a colon and the I/O kind (`not_found`), and whose
`at` is -1. It is read like the derives, when the program names it. *(ADR 0774)*

**json.load.2** — `print_json[T ToJson](v T)` prints `v.to_json().to_text()` and a
newline. *(ADR 0774)*

```ail rules=json.load.1,json.load.2
struct U
    name str
    age int

impl ToJson for U
impl FromJson for U

fn main() !
    path := "json_load_spec.tmp"
    fs.write_text(path, "[\{\"name\": \"al\", \"age\": 30}]")?
    users := load_json[List[U]](path)?
    print_json(users)                     # [{"name":"al","age":30}]
    fs.remove(path)?
    gone := load_json[U](path) else err
        print(err.message)                # json_load_spec.tmp: not_found
        users[0]
    print_json(gone)
```

**json.derive.1** — `trait ToJson` has `fn to_json(self) Json`, with the
generator `derive_to_json` (`ct.derive.1`). `str`, `int`, `float`, `bool`,
`Json` and `List[T ToJson]` implement it, and `Map[K, V]` and `T?` do when
their values do (`json.derive.4`). A derived struct gives an object of
its fields in declaration order, a `T?` field that is `none` giving `null`; a
derived enum gives a variant without fields as its name, and one with fields
as an object `{"kind": "Name", fields...}`. Fields of type `Map[str, T]`,
`List[T]` and `T?` convert element by element when `T` has no `to_json`
(`List[int?]`, `Map[str, List[U]]`), and a tuple field gives an array of its
elements (`type.tuple.8`). A generic type derives it through
bounds on the `impl`: `impl ToJson for Pair[A ToJson, B ToJson]`.
*(ADR 0308, ADR 0343, ADR 0795)*

**json.derive.2** — `trait FromJson` has the associated function
`fn from_json(j Json) Self ! JsonError` (`trait.assoc.1`); its generator
`derive_from_json` fills an empty `impl` with it, and `str`, `int`,
`float`, `bool`, `Json` and `List[T FromJson]` implement it. It is called as
`T.from_json(j)` (`trait.assoc.4`), in generic code too (`trait.assoc.2`); a
list is read with `List[U].from_json(j)` (ADR 0748). A
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
error. A tuple field reads an array of as many elements (`type.tuple.8`). A
map whose key isn't `str` and any other type (`Set[T]`) are a `ct.derive.4`
error naming the type. *(ADR 0308, ADR 0343, ADR 0350, ADR 0795)*

**json.derive.3** — A failure of a derived `from_json` names its path from
the document root: `missing items[2].kind` for a missing member, and
`items[2].kind: expected a string, found number` or ``shape: unknown Shape
`Box` `` for the others. `e.under(part)`, with `part` a member name or
`[i]`, puts a `JsonError` under one more step of its path. *(ADR 0343)*

```ail rules=json.derive.1,json.derive.2,json.derive.3
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

**json.derive.4** — `Map[K, V]` implements `ToJson` when `V` does, and `T?`
when `T` does, with no `impl` block, as `T?` is `Eq` when `T` is
(`trait.auto.1`). A map gives an object of its entries in insertion order,
each member named by its key's text (`"{k}"`, so the `int` key `404` gives
`"404"`); `none` gives `null` and a present value its own JSON. So
`print_json(m)`, `m.to_json()` and a `[T ToJson]` bound take a map or an
optional at any depth: `List[int?]`, `Map[str, List[U]]`. A map or optional
whose values aren't `ToJson` is a `gen.bound.3` error naming the value
type. *(ADR 1093)*

```ail rules=json.derive.4
struct Stats
    count int

impl ToJson for Stats

fn main()
    m := {"get": Stats(count: 3)}
    print_json(m)                     # {"get":{"count":3}}
    print_json(m.get("put"))          # null
    print_json([m.get("get"), m.get("put")])  # [{"count":3},null]
```

## Availability

**json.prelude.1** — The items above are read only when the program mentions
`Json`, `JsonMember`, `JsonError`, `parse_json`, `ToJson` or `FromJson`. A
program that declares one of those names itself gets its own and none of the
module's. The other prelude modules work the same way, one at a time: a
program with its own `struct Heap` gets no prelude `Heap` but still gets
`Grid`, `Deque` and the JSON items. *(ADR 0162, ADR 0308)*

```ail rules=json.prelude.1
fn main()
    print(Json.Null.to_text())
```

### `json.pointer` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on json.pointer.1

Any other path starts with `/` and names one step per `/`-separated token: `~1` stands for `/` and `~0` for `~` (in that order), a token is a member key of an object, and an array index of an array (digits only, no leading zero; `-` is past the end). It is `none` when the path doesn't start with `/`, a member or index is missing, or a step lands on a value that is not a container. A key repeated in an object is the first.
