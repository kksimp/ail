# Self-hosting status (Stage 3)

> **Non-normative.** Where the AIL-written front end stands against the
> Rust bootstrap (ADR 0022, ADR 0032). Lane 113 writes the lexer and parser,
> lane 114 the checker; both are compared with the Rust ones file by file.

## Layout

| Path | What | ADR |
|---|---|---|
| `compiler/ast/` | The syntax tree, shared; its derived JSON is the shape of `ail api --ast` | [0376](../decisions/0376-syntax-tree-in-ail-and-its-json.md) |
| `compiler/syntax/` | Lexer and parser (lane 113) | 0371–0375 |
| `compiler/check/` | Checker (lane 114) | 0376–0380 |

## Front-end agreement

`compiler/selfhost.ail tokens|parse FILE...` against `ail api --tokens`
and `ail api --ast` on every `.ail` file in the repository, byte for byte;
diagnostics are a rule and a position (ADR 0373). The selftest runs both
(`bootstrap/tests/self_hosting.rs`).

| Stage | Source | Files | Agreement |
|---|---|---|---|
| Lexer | `compiler/syntax/lex.ail` | 770, and 700 fuzzed | all identical |
| Parser | `compiler/syntax/parse.ail`, `recover.ail` | 770, and 1,000 fuzzed | all identical |

## Checker agreement

Diagnostics of `ail check` (Rust) and the AIL checker on every program and
corpus mistake: same rule, line and column, in order. Filled per slice.

| Slice | Scope | Agreement |
|---|---|---|
| 1 | Names, types, prelude | not started |
| 2 | Expressions and statements | not started |
| 3 | Ownership and effects | not started |

## Language gaps found writing the compiler in AIL

| Gap | State |
|---|---|
| Derived `from_json` built variants with two fields of one type positionally (`type.struct.7`) | Fixed (ADR 0376) |
| `impl ToJson`/`FromJson` for a type in a module failed (`ct.derive.2`) | Fixed (ADR 0376) |
| A prelude local clashed with a program function of its name (`expr.bind.5`) | Fixed (ADR 0376) |
| `unique`, `par`, `ensures` can't be field names, though a field name is never ambiguous with them | Open; renamed `is_*` for now |
| A variant can't have a field `kind` when its enum derives JSON | By design (json.derive.2); renamed `shape`, `form` |
| Derived JSON writes integers as floats (`1.0`), exact only below 2^53 | Worked around: integer literals as text |
| No way to write a typed `none` for a mutable local (`mut x := none` has no type) | Worked around: `absent[T]()`, a generic function returning `none` |
| `if c then x else none` with a plain `T` for `x` doesn't infer `T?` (`type.opt.4`) | Worked around: a helper function |
| A local can't take the name of any item in its module (`expr.bind.5`); with a module of several files, a helper `word` in one file forbids a local `word` in all | Open |
| The Rust lexer continued names with any Unicode numeric (`x٣`); AIL's `is_alnum` takes ASCII digits only | Fixed: names are letters, ASCII digits, `_` (ADR 0372) |
