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

## Runtime traces: `ail run --trace` (ADR 0265)

Spec chapter [21-trace](../spec/21-trace.md) (`trace.*`) says what is traced;
this section is the format. `--trace` (or `--trace=all`), `--trace=calls` or
`--trace=effects` writes one event per line to **stderr**; standard output
stays the program's. Human lines:

```
trace: call main() at 9:4
trace:   call fact(n = 3) at 10:11
trace:   contract fact requires `n >= 0`: held
trace:   fact returned 6
trace: effect fs.exists("data.txt") at 9:11
trace: spawn square(7) at 11:10
trace: wait Task#1 at 12:13
watch: total = 3 at 12:9
trace: summary: 11 events shown, 0 dropped (call 4, return 4, contract 3)
```

Calls indent two spaces per depth. Positions in the program's first file are
`LINE:COL`; other files are `FILE:LINE:COL`. With `--json`, one object per
event, keys in this order:

| `trace` | Other keys |
|---|---|
| `call` | `fn`, `depth` (`main` is 1), `args` (object: name to value text), position |
| `return` | `fn`, `depth`, `value` |
| `contract` | `fn`, `kind` (`requires`/`ensures`), `cond`, `held` (bool), position of the contract |
| `effect` | `call` (`fs.read_text`), `args` (array of value texts), position |
| `spawn` | `task` (`f(args)`), position |
| `wait` | `task` (`Task#N`), position |
| `watch` | `name`, `value`, position |
| `summary` | `shown`, `dropped`, then one count per kind that occurred |

Every object starts `"api":1,"trace":KIND`. A position is `"line","col"`,
preceded by `"file"` outside the first file. Values are text forms built
without user code (`trace.val.1`), cut at 60 characters (1,000 for `watch`)
with `…` (`trace.val.2`). `--trace-limit N` (default 1,000) caps the events
shown; the summary counts the rest.

```json
{"api":1,"trace":"call","fn":"fact","depth":2,"args":{"n":"3"},"line":10,"col":11}
{"api":1,"trace":"summary","shown":11,"dropped":0,"call":4,"return":4,"contract":3}
```

`--watch NAME` adds a `watch` event after each assignment rooted in a variable
`NAME` and each `mut NAME := e` (ADR 0268). It works with or without `--trace`.

## Trap snapshots (ADR 0267)

A trap that ends `ail run` keeps its first line and adds a snapshot, at most
ten lines (`trace.snap.3`):

```
x.ail:9:11: trap[expr.arith.3]: division by zero in `0 / 0`
  9 |     total / count
    |           ^
  values: amount = 25, balance = 10        (failed contracts only)
  at 9:11 in average(xs = []); total = 0, count = 0
  at 14:11 in main(); scores = [3, 4]
  … 16 more frames (--verbose shows every frame and value)
```

Each frame line is `at POSITION in FUNCTION(PARAMETERS); OTHER LOCALS`,
innermost first. `--verbose` prints every kept frame (up to 64) and each local
on its own line at the 1,000-character budget. With `--json`, the trap is one
object on stderr:

```json
{"api":1,"severity":"trap","rule":"expr.arith.3","file":"x.ail","line":9,"col":11,"message":"division by zero in `0 / 0`","source":"    total / count","stack":[{"fn":"average","file":"x.ail","line":9,"col":11,"args":{"xs":"[]"},"locals":{"total":"0","count":"0"}},{"fn":"main","file":"x.ail","line":14,"col":11,"args":{},"locals":{"scores":"[3, 4]"}}],"omitted_frames":0}
```

| Key | Always | Meaning |
|---|---|---|
| `severity` | yes | `"trap"` |
| `rule`, `file`, `line`, `col`, `message` | yes | As in the human first line |
| `source` | no | The text of the trap's line (absent in the prelude) |
| `condition` | no | A failed contract's names and values |
| `stack` | yes | Frames innermost first: `fn`, position, `args`, `locals` (objects: name to value text) |
| `omitted_frames` | yes | Frames past the 64 kept |

The editor (ADR 0241) can run `ail run --json --trace` and show these
objects in its run panel.

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

### Program card

`ail api --card [--budget N] [--json] DIR|FILE` prints a program's own card
(ADRs 0310–0311).

**api.card.1** — The card of a package directory or a file has, in this
order: its name; `entry:` the `main` of the root module and its file;
`tests:` the `@test` functions' names, sorted; the lines of `ail.pkg`;
`examples:` the `.ail` files under `DIR/examples`, each with the first line
of its first `##` comment; then one block per module, the root module
first, then by import path, listing the module's declarations in source
order as `ail api --text` does (signatures, with `pub` where written),
each function's effects in `[...]`, its `requires`/`ensures` and, for an
inferred error set, `fails with`, and the first line of its doc after `##`.
`@test` functions are not in the module blocks.

**api.card.2** — With `--json` (or the global `--json`) the same content is
one JSON line: `api`, `card`, `entry`, `tests`, `test_count`, `manifest`,
`examples`, `modules` (each `module`, `count`, `items` with `kind`, `name`,
`pub`, `file`, `line`, `col`, `signature`, `doc`, `effects`, `requires`,
`ensures`, `fails_with`, `members`), `budget` and `dropped`. `file`, `line`
and `col` are where the declaration's name is, the file as the program
loaded it ([ADR 0338](../decisions/0338-card-json-positions.md)); the text
form doesn't show them.

**api.card.3** — The card fits `--budget` estimated tokens (default 4,000).
If the full card is over, it drops in this order until it fits: private
items; doc comments; parameters' names; fields, variants, contracts, error
sets and test names; signatures (kind and name only, then module names
with counts). A trimmed card's last line is `trimmed to fit N tokens (~M);
dropped: LIST`.

**api.card.4** — A program the checker rejects has no card: the
diagnostics are printed and the exit code is 2, as for api.out.5. A missing
path or a bad option is exit code 2.

**api.card.5** — `docs/design/cards/edit.md` and `std.md` are the output for
`tools/edit` and `bootstrap/src/std`; `cargo test --test api_card` fails
when either differs from the command's output. Regenerate with
`./ail api --card tools/edit > docs/design/cards/edit.md`.

### Position queries: `--locals`, `--type`, `--refs`

`ail api --locals FILE LINE COL`, `ail api --type FILE LINE COL` and
`ail api --refs FILE LINE COL` answer a question about one position of the
file ([ADRs 0258–0261](../decisions/0258-api-locals-query.md)). They take
`--pretty` and `--json` as above, and the same program-with-errors rule
(api.out.5). Every answer starts `{"api":1,"file":F,"line":L,"col":C`.

**api.query.1** — A position with nothing to answer (`--type` or `--refs` off
a name, or a name with no recorded type or symbol) exits 1 and prints on
stderr one object `{"api":1,"severity":"error","message":...}`. A missing or
non-numeric `LINE COL` is a usage error, exit 2.

**api.loc.1** — `--locals` adds `locals`: every name in scope at the
position, as `{name, type, mut, kind, line, col}`. `kind` is `self`, `param`,
`loop` or `binding`; `line` and `col` are where the name is written (`null`
for `self`). The order is `self`, then by where each name is written. `type`
is the checker's, as in a diagnostic, or `"?"` where it is not known.

**api.loc.2** — A `:=` binding is in scope from the line after it. A
parameter, loop variable or pattern name is in scope from the first
statement of the block it scopes. A name stays in scope until a line indented
less than the line it started on; blank and comment lines never end a scope.
On an empty line the cursor's column gives the indent.

**api.type.1** — `--type` finds the name or number under (or just before) the
cursor and adds `text`, `start_col`, `end_col` (exclusive), `type` (the
innermost expression that starts there, or the type of a binding site) and
`symbol` (the id of the declaration it refers to, as `--symbols`, or `null`).
A qualified use of a module item (`geo.origin`) is one token at `geo`.

**api.type.2** — `callee` is the declaration's outline item when `symbol` is
a function or method (`signature`, `params`, `returns`, `effects`,
`requires`, `ensures`, `doc`; api.out.3), else `null`.

**api.refs.1** — `--refs` adds `symbol` and `refs`, a list of
`{file, line, col, def}` sorted by file order, line and column. `symbol` is
the id of the declaration the cursor is on or in a use of (api.ref.1) or
`local:NAME`. The definition has `def:true`; `file` is the source file of the
module the position is in, so a reference in an imported user module names
that module's file.

**api.refs.2** — A use of a module item is reported at the start of its
qualified name (`geo.origin` at the `g`). A method call through a trait refers to the trait's
method (api.ref.2).

**api.refs.3** — For a local, `refs` is its binding (`def:true`) and every
use of the name within its scope (api.loc.2). Locals can't be shadowed
(expr.bind.4), so the name identifies the binding. Only locals of the file
asked about are supported.

### Package directories

**api.pkg.1** — `ail api [--outline] [--symbols] [--text|--pretty] DIR` outlines
a whole package: the root module's files and every module they import
([ADR 0325](../decisions/0325-ail-api-on-package-directories.md)). The object
has `api`, `package` (the path as given), `items` and, with `--symbols`,
`references`. Items are ordered by file (root files first, then modules in
load order) and line. Each item has the single-file keys, with its `id` prefixed
by its module (`fn:geo.origin`, `type:geo.P`, `method:geo.P.shift`; api.id.1),
and ends with `module` (the import path, empty for the root) and `file`.
`line` counts from the top of the item's file. A reference has `id`, `line`,
`col` and `file`. `--text` prints a `# FILE` line before each file's items.

**api.pkg.2** — `--locals|--type|--refs DIR FILE LINE COL` asks about `FILE`
in the context of the whole package `DIR`, so `--refs` lists uses in every file
of every module, whichever file declares or uses the symbol. `FILE` is a root
file of `DIR` or a file of a module the root imports. A file the package does
not load is an error (`api.pkg.2`, exit 2). `--locals` and locals in `--type` and
`--refs` need a root file (api.refs.3).

**api.pkg.3** — The cursor may be on the name after a module qualifier
(`geo.origin` at `origin`) as well as on the qualifier (api.refs.2); both ask
about the same symbol. This holds for `--refs` with or without `DIR`.

**api.pkg.4** — The single-file forms and their JSON are unchanged (api.out.1,
api.refs.1). `refs` always names the file each position is in.

```json
{"api":1,"file":"t.ail","line":14,"col":9,"locals":[{"name":"items","type":"List[Point]","mut":false,"kind":"param","line":10,"col":10},{"name":"sum","type":"int","mut":true,"kind":"binding","line":11,"col":9}]}
```

`--symbols` also lists fields and variant fields with their `type`
(api.out.3), so an editor can complete members without a position query.

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
