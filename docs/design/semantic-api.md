# The semantic API

Machine-readable views of a program for AI agents and tools (plan §2,
[ai-first-plan.md](ai-first-plan.md)). Every output is byte-stable: the same
program gives the same bytes, keys in a fixed order, items sorted by
position. Rules here use the `api` key and are cited by the bootstrap like
spec rules.

## Diagnostics

*(Diagnostics schema v1 is written by lane 1; this heading is a placeholder
so both sections can merge.)*

## ail api

`ail api FILE` prints a program's declarations without their bodies, so an
agent can understand a file or a dependency from a few hundred tokens
([ADR 0150](../decisions/0150-ail-api-outline.md)). `--symbols` adds every
reference ([ADR 0151](../decisions/0151-stable-symbol-ids-and-references.md)).
`--text` prints a compact outline instead of JSON.

### Output

**api.out.1** — The output is one JSON object with the keys `api` (the
shape's version, `1`), `file` (the path as given) and `items`, then
`references` with `--symbols`.

**api.out.2** — The object is pretty-printed with a two-space indent, every
object's keys in the order this section lists them, an empty array as `[]`,
and nothing but a newline after the closing `}`.

**api.out.3** — `items` holds every declaration of the file, sorted by
position. Each item has `kind`, `name`, `id`, `line`, `col`, `doc` (the doc
comment, or `null`) and `signature` (the declaration's first line as
written, without its body; a constant's value only when it is a literal,
else `...`). Then, by kind:

| `kind` | Further keys |
|---|---|
| `fn` | `params` (`name`, `type`, `mut`), `returns` (type text or `null`), `effects` |
| `method` | `receiver` (`"self"`, `"mut self"` or `null` for an associated function), then as `fn`; in a trait, also `default` (whether it has a body) |
| `struct` | `error`, `fields`, `methods` |
| `enum` | `error`, `variants`, `methods` |
| `trait` | `methods` |
| `impl` | `trait`, `type` (both ids), `methods` |
| `const` | none |
| `field` | `type` |
| `variant` | `fields` |

Fields, variants and members are nested items with the same common keys.
A method declared `fn Type.name` sits under its type when the file declares
that type, otherwise at the top level. Types are written as in source
(`List[any Shape]`, `Size?`, `int ! ParseError`).

**api.out.4** — `effects` lists the effects a function performs (plan §3).
Until effects land it is always `[]`; the key is present so consumers don't
change when it fills.

**api.out.5** — A program with errors has no outline: `ail api` reports the
diagnostics as `ail check` would (with `--json`, as JSON lines) and exits 2.
A file without `main` (mod.main.1) is not an error here, so a library
file has an outline. Warnings are not reported and don't stop the outline.

### Stable ids

**api.id.1** — An id names a declaration by its path, never by its
position, so it survives edits elsewhere in the file:

| Declaration | Id |
|---|---|
| function | `fn:add` |
| struct or enum | `type:Point` |
| trait | `trait:Shape` |
| method or associated function (`fn Point.shift`, or in a trait or `impl`) | `method:Point.shift`, `method:Shape.area` |
| field of a struct | `field:Point.x` |
| variant | `variant:Color.Red` |
| field of a variant | `field:Shape.Rect.w` |
| `impl Trait for Type` | `impl:Type.Trait` |
| constant | `const:MAX_DEPTH` |

**api.id.2** — When a method of an `impl` block would get the same id as
another method of the type, its id also names the trait:
`method:Circle.Shape.area`.

**api.id.3** — An id may name a standard declaration that isn't in the file,
such as `trait:Ord`.

### References

**api.ref.1** — With `--symbols`, `references` lists every use of a declared
name as `{id, line, col}`, sorted by position then id: calls, method calls,
field reads, struct-literal and variant labels, variants (qualified, bare
and in patterns), constants, and type and trait mentions in signatures,
`impl` headers, bounds and type arguments. A local variable, a declaration
itself and a use inside the standard prelude are not references.

**api.ref.2** — A method call that is dispatched through a trait refers to
the trait's method (`method:Shape.area`), not to an implementation: which
implementation runs is known only from the value.

### Doc comments

**api.doc.1** — A doc comment (lex.comment.2) is a run of `##` lines with
nothing between it and the declaration's line. A blank line or a plain `#`
comment ends a run and leaves it unattached.

**api.doc.2** — The doc is the lines' text after `##` and one optional
space, joined with newlines. Structs, fields, enums, variants, traits,
functions, methods and constants take docs; an `impl` block doesn't.

### Text outline

**api.text.1** — `--text` prints one declaration per line: its line number
right-aligned in five columns, two spaces, four spaces per nesting level,
and its signature. Each doc line follows, indented one level deeper as
`## text`.

```text
    2  trait Shape
    3      fn area(self) int
    8  struct Circle
    9      r int
   40      fn Drawing.add(mut self, s any Shape)
```

### Example

For

```ail
## A point.
struct Point
    x int
    y int

fn Point.shift(mut self, dx int)
    self.x += dx
```

`ail api` prints (abridged):

```json
{
  "api": 1,
  "file": "point.ail",
  "items": [
    {
      "kind": "struct",
      "name": "Point",
      "id": "type:Point",
      "line": 2,
      "col": 8,
      "doc": "A point.",
      "signature": "struct Point",
      "error": false,
      "fields": [
        {
          "kind": "field",
          "name": "x",
          "id": "field:Point.x",
          "line": 3,
          "col": 5,
          "doc": null,
          "signature": "x int",
          "type": "int"
        }
      ],
      "methods": [
        {
          "kind": "method",
          "name": "shift",
          "id": "method:Point.shift",
          "line": 6,
          "col": 10,
          "doc": null,
          "signature": "fn Point.shift(mut self, dx int)",
          "receiver": "mut self",
          "params": [
            {
              "name": "dx",
              "type": "int",
              "mut": false
            }
          ],
          "returns": null,
          "effects": []
        }
      ]
    }
  ]
}
```
