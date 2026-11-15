# Self-hosting status (Stage 3)

> **Non-normative.** Where the AIL-written front end stands against the
> Rust bootstrap (ADR 0022, ADR 0032). Lane 113 writes the lexer and parser,
> lane 114 the checker; both are compared with the Rust ones file by file.

## Layout

| Path | What | ADR |
|---|---|---|
| `compiler/ast/` | The syntax tree, shared; its derived JSON is the shape of `ail api --ast` | [0376](../decisions/0376-syntax-tree-in-ail-and-its-json.md) |
| `compiler/syntax/` | Lexer and parser (lane 113) | 0371–0375 |
| `compiler/check/` | Checker (lane 114) | [0377](../decisions/0377-checker-in-ail-ported-pass-for-pass.md), [0378](../decisions/0378-checker-gaps-are-skipped-not-guessed.md), [0740](../decisions/0740-self-hosted-checker-parity-gate.md) |

## Front-end agreement

`ail run compiler tokens|parse FILE...` against `ail api --tokens`
and `ail api --ast` on every `.ail` file in the repository, byte for byte;
diagnostics are a rule and a position (ADR 0373). The selftest runs both
(`77592931:bootstrap/tests/self_hosting.rs`).

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

Parsing `tools/edit/edit.ail` (5,249 lines) in the interpreter takes about
0.45 s to lex and 0.85 s to parse, measured inside the program. The
bootstrap lexes and parses the same file, writes its JSON dump and exits in
0.15 s, so the AIL front end is at least 8 times slower. The bytecode tier
(ADR 0361) doesn't change this yet: it compiles 119 of the 131 functions
called, but string, list and method operations hand off to the walker, 4.3
million times per parse.

## Checker agreement

Diagnostics of `ail check` (Rust) and the AIL checker on every program the
front end accepts: same rule, file, line and column, in order. Slices 1–3
compared only the rules and lines the AIL checker covered
(`compiler/check/coverage.ail`, ADR 0377). Since slice 4 the selftest
`self_hosted_check` compares with nothing skipped, except the package
manifest checks eff.cap.4–5 (ADR 0740).

| Slice | Scope | Agreement |
|---|---|---|
| 1 | Names, types, prelude: the declaration passes, 42 rules outside function bodies | 491 of 491 programs (2026-10-20) |
| 2 | Expressions and statements: every function body, minus gaps (ADR 0378) | 495 of 495 programs; types 1,819 of 1,848 bodies, 98.4% (2026-10-20) |
| 3 | Flow passes: use after move, task handles, effects (`@pure`, pure contracts, constants), parallel loops and `par for` comprehensions | 495 of 495 programs, 156 rules (2026-10-20) |
| 4 | Differential report, nothing skipped: `std.tensor`, `std.ui` and named colors ported, and the corpus widened to perfbench, the agent corpus, `tools/edit` and `compiler` | 522 of 522 programs; 2,615 of 2,615 bodies; 536 of 536 diagnostics, all 536 messages identical (2026-10-20) |
| 4+ | Ported lane 141's rules (ADRs 0468–0469): `mut x := none` typed by its first assignment (type.opt.9), an `if` with a bare `none` branch is `T?` (type.opt.10), a local may shadow a function | 528 of 528 programs; 540 of 540 diagnostics (2026-10-20) |
| 4+ | Ported the batch branch's checker changes: match guards and range patterns (ADRs 0452–0453, expr.match.11–12), `unsafe` blocks and functions and C calls (unsafe.block.1–3, ffi.call.1), bit methods on `int`, tensor `relu`/`sigmoid`/`tanh` and `/` (std.tensor.10, std.tensor.13); `unsafe_allowed` added to the program dump | 579 of 579 programs; 2,960 bodies; 557 of 557 diagnostics, all messages identical (2026-10-20) |
| 4+ | Ported named arguments (lane 130, expr.call.3): `Sig.names`, `arg_slots` and `slot_of` in `check_call` | 581 of 581 programs; 563 of 563 diagnostics, all messages identical (2026-10-20) |
| 4+ | Ported function values (lane 130, type.fn.1–3): `fn_value`, `call_value`, `FnTy` in `bind`, `subst_unknown` and `has_param`, and the effect of a function named as a value | 584 of 584 programs; 570 of 570 diagnostics, all messages identical (2026-10-20) |
| 4+ | Ported the second batch merge: `bytes` (type.bytes.2–6, 13; its methods and `str.to_bytes`), raw pointers and `std.mem` (unsafe.op.1, unsafe.ptr.1–3, unsafe.mem.3), `extern c fn` declarations (ffi.decl.3), `std.net` handles (std.net.9, `net.ail`), `task.select` (conc.select.1), tensor indexing, `-t` and the new tensor methods (std.tensor.14–19), the new `std` functions (hash, compress, regex, tz, proc, net), and effects per standard function (`net.serve` capability) | 668 of 668 programs; 3,990 bodies; 675 of 675 diagnostics, all messages identical, with the `net.Response` `all_headers` default (std.net.17) (2026-10-20) |

### Slice 4 report (2026-10-20)

| Corpus | Programs |
|---|---|
| `examples/` | every `.ail` |
| `tests/cases/`, including `packages/` | every case the front end accepts |
| `tests/mistakes/` (the `ail fix` corpus) | 140 |
| `tools/perfbench/programs/` | 6 |
| `tools/tokbench/corpus/programs/` (agent corpus) | 20 |
| `tools/edit`, `compiler` | 2 |

- **Agreement:** 100% of programs (522), bodies (2,615) and diagnostics
  (536), with every message identical.
- **Gaps left:** none. The last ones were `std.tensor` (functions, methods,
  operators, `tensor.Tensor(...)`), `std.ui` window methods and named
  colors, the hidden-module check std.use.4, and `print`'s wording for a
  function with no result.
- **Outside the checker:** eff.cap.4–5 (`77592931:bootstrap/src/packages.rs`), which
  come from package manifests and aren't in the AST dump; 5 diagnostics, all
  in `packages/caps_denied`.
- **Size:** `compiler/check/` is 11,000 lines of AIL, against 11,551 lines of
  Rust in `77592931:bootstrap/src/check*`.
- **Speed:** checking `compiler` and `tools/edit` (7 MB of AST JSON) takes
  about 15 s under the tree-walking interpreter, against under 1 s for
  `ail check`. This is the next thing to bring down, through the bytecode
  tier (ADR 0359) and then native code.

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
| A catch-all `_` over an enum always warns (expr.match.3), with no way to suppress it (ADR 0056 left the syntax open); the checker's code had 167 | Fixed: `@allow("expr.match.3")` (ADR 0379) |
| `pub` can't precede an attribute (`pub @allow(...) fn`, mod.vis.1) | Open; such helpers aren't `pub` |
| A match arm can't be empty (`_ => {}` is a map literal) | Open; `_ => return`, or a helper |
| `x?.y == true` and `if x?.y` need `else` (`(x?.y else false)`) | Open; written with `else` |
| A method's local can't share a name with a field of its receiver type (trait.method.7) | Allowed since [ADR 1121](../decisions/1121-locals-may-shadow-functions-and-modules.md) |
| No literal for an empty optional of a given type (`Ty?()` isn't valid) | Fixed: `mut x := none` is typed by its first assignment, [ADR 0468](../decisions/0468-optional-none-locals-and-if-branches.md) (type.opt.9); a local read before its first assignment still needs `absent[T]()` |
| No bit shifts (`<<`) | Open; constants instead |
| `if c then x else none` with a plain `T` for `x` doesn't infer `T?` (`type.opt.4`) | Fixed: [ADR 0468](../decisions/0468-optional-none-locals-and-if-branches.md) (type.opt.10) |
| A local can't take the name of any item in its module (`expr.bind.5`); with a module of several files, a helper `word` in one file forbids a local `word` in all | Fixed: [ADR 0469](../decisions/0469-locals-may-shadow-functions.md) (a local may shadow a function) |
| `if` can't be an operand, even in parentheses (`col + (if raw then 4 else 3)`) | Fixed: [ADR 0470](../decisions/0470-parenthesised-if-operand.md) (expr.if.8) |
| The Rust lexer continued names with any Unicode numeric (`x٣`); AIL's `is_alnum` takes ASCII digits only | Fixed: names are letters, ASCII digits, `_` (ADR 0372) |
