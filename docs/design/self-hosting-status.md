# Self-hosting status (Stage 3)

> **Non-normative.** Where the AIL-written front end stands against the
> Rust bootstrap (ADR 0022, ADR 0032). Lane 113 writes the lexer and parser,
> lane 114 the checker; both are compared with the Rust ones file by file.

## Layout

| Path | What | ADR |
|---|---|---|
| `compiler/ast/` | The syntax tree, shared; its derived JSON is the shape of `ail api --ast` | [0376](../decisions/0376-syntax-tree-in-ail-and-its-json.md) |
| `compiler/syntax/` | Lexer and parser (lane 113) | 0371–0375 |
| `compiler/check/` | Checker (lane 114) | [0377](../decisions/0377-checker-in-ail-ported-pass-for-pass.md) |

## Checker agreement

Diagnostics of `ail check` (Rust) and the AIL checker on every program and
corpus mistake the front end accepts: same rule, file, line and column, in
order, over the rules and lines the AIL checker covers so far
(`compiler/check/coverage.ail`, ADR 0377). The selftest
`self_hosted_check` requires full agreement.

| Slice | Scope | Agreement |
|---|---|---|
| 1 | Names, types, prelude: the declaration passes, 42 rules outside function bodies | 491 of 491 programs (2026-10-20) |
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
| A module's locals clashed with the root module's items (`expr.bind.5`) | Fixed (ADR 0377) |
| An or-pattern can't bind names (`Struct(_, xs) \| Enum(_, xs) => ...`, expr.match.8) | Open; arms repeated |
| A catch-all `_` over an enum always warns (expr.match.3), with no way to suppress it (ADR 0056 left the syntax open); the checker's code has dozens | Open |
| No literal for an empty optional of a given type (`Ty?()` isn't valid) | Open; a helper `fn no_ty() Ty?` |
| No bit shifts (`<<`) | Open; constants instead |
| The Rust lexer continued names with any Unicode numeric (`x٣`); AIL's `is_alnum` takes ASCII digits only | Fixed: names are letters, ASCII digits, `_` (ADR 0372) |
