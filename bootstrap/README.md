# AIL bootstrap

The Rust front end and reference interpreter for AIL ([ADR 0023](../docs/decisions/0023-rust-bootstrap.md),
[ADR 0032](../docs/decisions/0032-keep-rust-interpreter-as-oracle.md)).
It is temporary scaffolding for the compiler, apart from the interpreter, which
stays as a permanent test oracle.

## Using it

From the repository root (it sets itself up on first run, per ADR 0071 and ADR 0087):

```sh
./ail run examples/hello.ail       # macOS / Linux
.\ail.ps1 run examples\hello.ail   # Windows (not yet tested on Windows)
./ail check --json FILE            # diagnostics as JSON lines
./ail selftest                     # the bootstrap's own tests
```

Exit status: `0` success, `1` the program failed or trapped, `2` the program
was rejected or the command was invalid.

## Pipeline

| File | Stage | Spec rules enforced |
|---|---|---|
| `src/lexer.rs` | Text to tokens, including INDENT/DEDENT | `lex.block.1–4`, `lex.stmt.1–3`, `lex.comment.*`, `lex.str.1–5`, `lex.num.1–3` |
| `src/parser.rs` | Tokens to syntax tree | `gram.fn.1–2`, `gram.syntax.1`, `expr.bind.1` |
| `src/check.rs` | Names, bindings, calls, types | `mod.main.1`, `mod.main.3`, `mod.unit.1`, `expr.name.1`, `expr.bind.1/4/5`, `expr.block.2`, `expr.call.1–2` |
| `src/interp.rs` | Runs checked programs | `mod.main.2–3`, `err.trap.3` |
| `src/diag.rs` | Text and JSON diagnostics | ADR 0002 (rule IDs), ADR 0024 |

All errors in a file are reported in one pass, sorted by position.

## Supported subset (milestone 1: Hello World)

- `fn` declarations with `name Type` parameters and a return type
- `name := value` bindings (immutable)
- calls, string literals (with all escapes), integer literals (all bases)
- types `str`, `int` (`i64`)
- `print` (prelude)

Anything else the spec defines is rejected with a **`bootstrap.unsupported`**
diagnostic that names the relevant spec rule. That's an implementation limit,
not a spec rule. The bootstrap never silently accepts a construct it doesn't
implement correctly.

## Adding a feature

1. The behaviour must already be specified, with rule IDs, in `docs/spec/`.
2. Implement it, citing the rule IDs in comments and diagnostics.
3. Add test cases in `tests/cases/`:
   - `NAME.ail` + `NAME.out` for programs that must run and print exactly `NAME.out`;
   - `NAME.ail` + `NAME.rules` for programs that must be rejected, one expected
     rule ID per line, in report order.
4. `./ail selftest` must pass on Linux, and `tools/verify.sh BRANCH` on macOS
   (ADR 0085).
