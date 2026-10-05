# AIL bootstrap

The Rust front end and reference interpreter for AIL ([ADR 0023](../docs/decisions/0023-rust-bootstrap.md),
[ADR 0032](../docs/decisions/0032-keep-rust-interpreter-as-oracle.md)).
It is temporary scaffolding for the compiler, apart from the interpreter, which
stays as a permanent test oracle.

## Using it

From the repository root (it sets itself up on first run, per ADR 0071 and ADR 0087):

```sh
./ail run examples/hello.ail       # macOS / Linux
./ail run examples/gradebook.ail
.\ail.ps1 run examples\hello.ail   # Windows (not yet tested on Windows)
./ail check --json FILE            # diagnostics as JSON lines
./ail selftest                     # the bootstrap's own tests
```

Exit status: `0` success, `1` the program failed or trapped, `2` the program
was rejected or the command was invalid. Warnings (such as `expr.match.3`) are
reported but don't stop a program from running.

## Pipeline

| File | Stage | Spec rules enforced |
|---|---|---|
| `src/lexer.rs` | Text to tokens, including INDENT/DEDENT and interpolated strings | `lex.block.1–4`, `lex.stmt.1–3`, `lex.kw.1–2`, `lex.comment.*`, `lex.str.1–5`, `lex.num.1–3` |
| `src/parser.rs` | Tokens to syntax tree | `gram.fn.1–2`, `gram.list.1`, `gram.generic.1`, `gram.syntax.1`, `type.struct.1–3`, `type.enum.1`, `expr.bind.1–3`, `expr.if.1`, `expr.if.3`, `expr.loop.1–3`, `expr.match.1`, `expr.prec.1–2`, `expr.op.1–3` |
| `src/check.rs` | Names, types, places, control flow | `mod.main.1`, `mod.main.3`, `mod.unit.1`, `expr.name.1`, `expr.bind.1–6`, `expr.block.2–3`, `expr.call.1–2`, `expr.if.2`, `expr.if.4`, `expr.loop.3–5`, `expr.match.2–5`, `expr.op.2–3`, `type.bool.1`, `type.coll.2–6`, `type.str.2–3`, `type.struct.3–6`, `type.enum.3`, `type.enum.5`, `trait.method.1–5`, `trait.op.1`, `trait.op.3`, `own.param.1–2`, `own.excl.1–2` |
| `src/interp.rs` | Runs checked programs | `mod.main.2–3`, `own.val.1–2`, `own.excl.3`, `expr.arith.1`, `expr.arith.3`, `expr.arith.5`, `expr.loop.4`, `type.coll.4`, `err.trap.1`, `err.trap.3` |
| `src/diag.rs` | Text and JSON diagnostics, errors and warnings | ADR 0002 (rule IDs), ADR 0024 |

All errors in a file are reported in one pass, sorted by position.

## Supported subset (milestone 2: gradebook)

- **Declarations:** `fn` with `name Type` and `mut name Type` parameters and a
  return type; `struct` with `name Type` fields; `enum` with variants that
  carry no data; methods `fn Type.name(self, ...)` and `fn Type.name(mut self, ...)`;
  associated functions `fn Type.name(...)`
- **Statements:** `x := v`, `mut x := v`, `p = v`, `p op= v` (for `+ - * / %`),
  `if`/`elif`/`else`, `while`, `for x in list`, `for i in a..b` and `a..=b`,
  `break`, `continue`, `return`
- **Expressions:** `if` and `match` as values; `match` on enums with variant and
  `_` patterns; calls, `mut` arguments, method calls, fields, indexing
  `xs[i]`, slices `xs[a..b]`; struct literals `Type(field: value)`; list
  literals; operators by ADR 0112's precedence; string interpolation `"{expr}"`;
  bare enum variants where the type is known
- **Types:** `int` (`i64`), `bool`, `str`, `List[T]`, structs, enums
- **Library:** `print`, `xs.len()`, `xs.push(v)`
- **Semantics:** value semantics with copy-on-write; `mut` operands are moved
  into calls and written back; overflow, division, index and overlap traps

Not yet: floats, `char`, optionals and errors (`T?`, `T ! E`, `?`), variants
with data, literal patterns, maps and sets, traits, generics, modules,
`defer`, `ref`, `unique`, tasks, `unsafe` and FFI. Text forms exist only for
`int`, `bool` and `str` until the formatting trait is designed.

Anything else the spec defines is rejected with a **`bootstrap.unsupported`**
diagnostic that names the relevant spec rule. That's an implementation limit,
not a spec rule. The bootstrap never silently accepts a construct it doesn't
implement correctly.

## Adding a feature

1. The behaviour must already be specified, with rule IDs, in `docs/spec/`.
2. Implement it, citing the rule IDs in comments and diagnostics.
3. Add test cases in `tests/cases/`:
   - `NAME.ail` + `NAME.out` for programs that must run and print exactly `NAME.out`;
   - `NAME.ail` + `NAME.rules` for programs with diagnostics, one expected rule
     ID per line in report order. The program must be rejected if any of them
     is an error, and run if all are warnings;
   - `NAME.ail` + `NAME.trap` (+ optional `NAME.out`) for programs that must
     print `NAME.out` and then trap with the rule ID in `NAME.trap`.
4. Keep the code formatted with `rustfmt` (settings in `rustfmt.toml`) and free
   of `clippy` warnings. Neither is in the minimal toolchain the designer
   uses, so agents add them locally (`rustup component add rustfmt clippy`).
5. `./ail selftest` must pass on Linux, and `tools/verify.sh BRANCH` on macOS
   (ADR 0085).
