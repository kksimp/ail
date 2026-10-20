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

## Front-end size and speed

Measured 2026-10-20 on Linux x86-64. Tokens are the mean of tokbench's five
tokenizers. The Rust count is `lexer.rs`, `parser.rs`, `parser/*.rs` and
`foreign.rs` without their test modules; the AIL count is `lex.ail`,
`parse.ail` and `recover.ail` (`dump.ail` corresponds to Rust's
`api/syntax.rs`, which isn't counted).

| | Rust | AIL | AIL vs Rust |
|---|---|---|---|
| Lexer: lines | 1,132 | 1,101 | −3% |
| Lexer: tokens | 12,674 | 10,651 | −16% |
| Parser: lines | 3,311 | 2,281 | −31% |
| Parser: tokens | 35,752 | 23,580 | −34% |
| Total tokens | 48,425 | 34,231 | −29% |

The Rust parser also writes the message, help and fixes of each
diagnostic, about a third of its source, which the AIL one doesn't yet
(ADR 0373). Without them the two are about the same size. The token saving
is in the lexer, which writes everything, and in the grammar: no lifetimes,
borrows or `Box`, and implicit `self`.

Parsing `tools/edit/edit.ail` (5,249 lines) in the tree-walking
interpreter takes 0.45 s to lex and 0.78 s to parse, measured inside the
program. The bootstrap lexes and parses the same file, writes its JSON
dump and exits in 0.15 s. So the AIL front end is at least 8 times slower
for now; the bytecode tier (ADR 0359) is what should close that gap.

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
| No way to write a typed `none` for a mutable local (`mut x := none` has no type) | Worked around: `absent[T]()`, a generic function returning `none`; [OQ-114](../open-questions.md) |
| `if c then x else none` with a plain `T` for `x` doesn't infer `T?` (`type.opt.4`) | Worked around: a helper function; [OQ-115](../open-questions.md) |
| A local can't take the name of any item in its module (`expr.bind.5`); with a module of several files, a helper `word` in one file forbids a local `word` in all | Open: [OQ-116](../open-questions.md) |
| `if` can't be an operand, even in parentheses (`col + (if raw then 4 else 3)`) | Worked around: two whole branches; [OQ-117](../open-questions.md) |
| The Rust lexer continued names with any Unicode numeric (`x٣`); AIL's `is_alnum` takes ASCII digits only | Fixed: names are letters, ASCII digits, `_` (ADR 0372) |
