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

Notes today: `mod.unit.1` on a type declared twice points at the first
declaration.

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

## Program outline: `ail api FILE`

*To be written by the `ail api` work (plan §2.3–2.4).*
