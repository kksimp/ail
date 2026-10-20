# Semantic API

> **Non-normative design note.** The machine-readable interfaces of the `ail`
> tool, for AI agents and other tools ([ADR 0024](../decisions/0024-semantic-api-from-stage-2.md),
> [ai-first-plan.md](ai-first-plan.md) §2). Every interface here is versioned
> by the `"api"` key; this document describes **api 1**.

## Versioning

Every JSON object the tool prints carries `"api":1` as its first key. Adding
an optional key keeps the version; removing, renaming or changing the meaning
of a key makes it `"api":2`. Output is byte-stable: the same input gives the
same bytes, in a fixed key order, sorted as stated below.

Positions are 1-based. Columns count **characters** (Unicode scalar values),
not bytes. A span's end (`end_line`, `end_col`) is **exclusive**: it is the
position just after the last character.

## Diagnostics: `ail check --json FILE` (ADR 0145)

`ail check --json` and `ail run --json` print diagnostics on **stderr**, one
JSON object per line, sorted by position (diagnostics at the same position
keep the order in which the checker found them). The exit status is 2 if any
diagnostic is an error, otherwise 0 (`run`: the program's own status).

```json
{"api":1,"severity":"error","rule":"expr.name.1","file":"x.ail","line":5,"col":5,"message":"unknown name `prnt`","help":"did you mean `print`?","end_line":5,"end_col":9,"fixes":[{"title":"replace `prnt` with `print`","edits":[{"line":5,"col":5,"end_line":5,"end_col":9,"text":"print"}]}]}
```

| Key | Type | Always | Meaning |
|---|---|---|---|
| `api` | int | yes | Schema version, `1` |
| `severity` | `"error"` or `"warning"` | yes | A warning doesn't reject the program |
| `rule` | string | yes | Spec rule ID (`docs/spec/`), or `bootstrap.unsupported`; `ail explain RULE` explains it |
| `file` | string | yes | The path as given on the command line |
| `line`, `col` | int | yes | Start of the span |
| `message` | string | yes | One sentence, no trailing newline |
| `help` | string | no | How to fix it, in words |
| `end_line`, `end_col` | int | yes | Exclusive end of the span; the end of the token at the start when the checker gives no end; equal to the start when there's no token |
| `notes` | array | no | Related positions: `[{line, col, message}]`, e.g. where a name was first declared |
| `fixes` | array | no | Machine-applicable fixes, best first: `[{title, edits}]` |

Each fix is applied **all or nothing**. `edits` is a list of
`{line, col, end_line, end_col, text}`: replace the text from `(line, col)`
up to, not including, `(end_line, end_col)` with `text`. An insertion has an
empty span; a deletion has empty `text`. Edits of one fix don't overlap and
are listed in source order, so applying them from last to first keeps the
earlier positions valid.

A fix is only reported when it was checked against the source text: the
checker anchors an edit on the text it expects to replace, and drops the fix
if that text isn't there.

Fixes today:

| Rule | Situation | Fix |
|---|---|---|
| `expr.name.1`, `type.struct.4`, `type.struct.6`, `trait.impl.5` | a name close to a known one ("did you mean") | replace the name |
| `expr.bind.4` | `x := e` or `mut x := e` again for a `mut` variable `x` | `x = e` |
| `expr.bind.3` | `y = e` with `y` undeclared | `mut y := e` |
| `expr.bind.3` | a `:=` binding that is assigned or mutated later | `mut ` before the name (`fix.edit.1`) |
| `expr.if.1` | `else if` | `elif` (`fix.edit.2`) |
| `err.use.1` | a fallible call as a statement or `:=` value, where the function's errors admit its own | `?` at the end of the line (`fix.edit.3`) |
| `expr.match.2` | a `match` misses variants (or `true`/`false`, or a `_`) | one arm per missing case, each `=> todo` (`fix.edit.4`) |
| `std.use.2` | a standard module used without `use std.NAME` | `use std.NAME` on line 1 (`fix.edit.5`) |

Notes today (`fix.note.1`): `mod.unit.1` on a type, function, method or
constant declared twice, `expr.bind.4` on a local declared twice, and
`expr.bind.5` on a local named like a function, type or constant, each point
at the first declaration.

An edit is always plain coordinates, but the checker may name two positions by
rule and let the tool resolve them against the source (a block's end, a
line's end; [ADR 0204](../decisions/0204-anchored-fix-edits.md)); a fix whose
anchor doesn't resolve isn't reported.

## ail fix: `ail fix FILE [--dry-run] [--json]` (ADR 0203)

Applies fixes so an agent doesn't have to (`fix.apply.1`–`4`): the first fix of
each diagnostic, skipping a fix that overlaps an earlier one, last to first;
then re-check, up to 5 rounds. A file that was in canonical layout is
formatted afterwards. `--dry-run` prints a diff and leaves the file.

Text: one `fixed FILE:L:C: [rule] title` line per fix, then
`applied N fix(es) in R round(s); E error(s) remain`, and the remaining
diagnostics on stderr. With `--json`, stdout is one object per fix and a
summary:

```json
{"api":1,"applied":{"round":1,"rule":"expr.bind.3","line":22,"col":5,"title":"declare `x` with `mut`"}}
{"api":1,"summary":true,"applied":1,"rounds":1,"dry_run":false,"clean":true,"errors":0}
```

Exit 0 if no error remains (warnings such as `expr.hole.1` don't count), 1 if
one does, 2 for a usage or read error.

The text form (without `--json`) shows the same information:

```
x.ail:5:5: error[expr.name.1]: unknown name `prnt`
        prnt(x)
        ^
help: did you mean `print`?
fix: replace `prnt` with `print`
```

## Rule explanations: `ail explain RULE [--json]` (ADR 0146)

Prints a spec rule from the specification embedded in the binary: its text,
the ADRs it cites (with titles), and the nearest `ail` example after it in
the same section. On stdout, exit 0.

```json
{"api":1,"rule":"expr.bind.4","file":"docs/spec/05-expressions.md","line":23,"text":"Declaring a local name ...","adrs":[{"id":"0039","title":"Declarations: ..."},{"id":"0044","title":"..."}],"example":"fn total(items List[Item]) i64\n    ..."}
```

| Key | Type | Always | Meaning |
|---|---|---|---|
| `api` | int | yes | `1` |
| `rule` | string | yes | The rule ID asked for |
| `file`, `line` | string, int | yes | Where the rule is in the repository |
| `text` | string | yes | The rule's Markdown text, without the ID prefix |
| `adrs` | array | yes | `[{id, title}]` in order of citation |
| `example` | string | no | The example's source, without the fence |

`ail explain bootstrap.unsupported` explains the implementation-limit code.
An unknown rule exits 2 with, on stderr, `error: unknown rule ...` and a
`help:` listing known rules nearby; with `--json`, one object
`{"api":1,"severity":"error","message":...,"help":...}`.

## Typed holes (ADR 0147)

`todo` in an expression is reported as a warning `expr.hole.1` whose message
has a fixed shape an agent can read:

```
hole: expected `T`; in scope: `a int`, `xs List[str]`
hole: the expected type isn't known here; in scope: nothing
```

Names are sorted; a local whose type isn't known shows as `` `n` (type unknown) ``.

## ail api

`ail api FILE` prints a program's declarations without their bodies, so an
agent can understand a file or a dependency from a few hundred tokens
([ADR 0150](../decisions/0150-ail-api-outline.md)). `--symbols` adds every
reference ([ADR 0151](../decisions/0151-stable-symbol-ids-and-references.md)).
`--pretty` indents the JSON for a human, and `--text` prints a compact
outline instead.

### Output

**api.out.1** — The output is one JSON object with the keys `api` (the
shape's version, `1`), `file` (the path as given) and `items`, then
`references` with `--symbols`.

**api.out.2** — By default the object is written on one line with no
spaces outside strings. `--pretty` writes it with a two-space indent and a
space after each `:`. Either way every object's keys are in the order this
section lists them, an empty array is `[]`, and a newline ends the output.

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

**api.out.4** — `effects` lists the effects the function performs, sorted, from the inferred effect set (`eff.infer.*`, [ADR 0163](../decisions/0163-inferred-effect-sets.md)): `fail`, `io`, `mutates`, `spawn`, `unsafe` and the capabilities `fs`, `env`, `proc`, `clock`, `net`. `ail effects FILE` prints the same sets on their own.

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
`## text`. A variant's fields are not repeated under it, since its
signature shows them.

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

`ail api --pretty` prints (abridged):

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
