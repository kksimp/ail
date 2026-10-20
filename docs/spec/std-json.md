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
`?` in a function that returns an optional. *(ADR 0161)*

## Availability

**json.prelude.1** — The items above are read only when the program mentions
`Json`, `JsonMember`, `JsonError` or `parse_json`. A program that declares one
of those names itself gets its own and none of the module's. *(ADR 0162)*
