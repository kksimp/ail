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
./ail run examples/commands.ail
./ail run examples/shapes.ail
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
| `src/prelude.ail` | The standard traits `Eq`, `Ord`, `ToStr` and `enum Ordering`, read before every program at line 0 | `trait.std.1–3` |
| `src/parser.rs` | Tokens to syntax tree | `gram.fn.1–2`, `gram.list.1`, `gram.generic.1`, `gram.syntax.1`, `gen.decl.1`, `gen.bound.1`, `gen.method.1–3`, `gen.infer.2`, `trait.decl.1–2`, `trait.impl.1`, `trait.any.1`, `type.struct.1–3`, `type.enum.1–2`, `type.opt.2`, `err.decl.1`, `err.type.1–2`, `err.type.5`, `err.else.1–3`, `expr.bind.1–3`, `expr.if.1`, `expr.if.3`, `expr.if.5–6`, `expr.loop.1–3`, `expr.loop.6`, `expr.match.1`, `expr.match.6`, `expr.prec.1–2`, `expr.op.1–3` |
| `src/check.rs` | Names, types, places, control flow | `mod.main.1`, `mod.main.3`, `mod.unit.1`, `expr.name.1`, `expr.bind.1–6`, `expr.block.2–3`, `expr.call.1–2`, `expr.if.2`, `expr.if.4`, `expr.loop.3–5`, `expr.match.2–5`, `expr.op.2–3`, `type.bool.1`, `type.coll.2–7`, `type.str.2–3`, `type.struct.3–6`, `type.enum.3`, `type.enum.5`, `trait.method.1–5`, `trait.op.1`, `trait.op.3`, `own.param.1–2`, `own.excl.1–2`, `own.excl.5`, `type.opt.4–5`, `type.enum.6`, `type.str.4–5`, `expr.loop.6`, `expr.match.6`, `err.decl.1–3`, `err.type.1–2`, `err.type.5–6`, `err.prop.1–3`, `err.use.1–2`, `err.else.1–3`, `err.ctx.2`, `err.text.1`, `expr.if.5–6`, `expr.cmp.1`, `gen.decl.1–2`, `gen.bound.1–4`, `gen.method.1–3`, `gen.infer.1–2`, `gen.anon.1`, `trait.decl.1–2`, `trait.impl.1–6`, `trait.self.1`, `trait.res.1`, `trait.any.1–5`, `trait.std.1–4`, `trait.derive.1`, `trait.auto.1–4` (traits and generics in `src/check/traits.rs`) |
| `src/interp.rs` | Runs checked programs | `mod.main.2–3`, `own.val.1–2`, `own.excl.3–4`, `expr.arith.1`, `expr.arith.3`, `expr.arith.5`, `expr.loop.4`, `expr.loop.6`, `type.coll.4`, `type.str.4–5`, `err.prop.1`, `err.else.1–3`, `err.ctx.1–3`, `err.text.1`, `err.trap.1`, `err.trap.3`, `gen.impl.1`, `trait.any.1`, `trait.std.1–4`, `trait.derive.1`, `trait.auto.1–2` |
| `src/diag.rs` | Text and JSON diagnostics, errors and warnings | ADR 0002 (rule IDs), ADR 0024 |

All errors in a file are reported in one pass, sorted by position.

## Supported subset (milestone 4: traits and generics)

- **Declarations:** `fn` with `name Type` and `mut name Type` parameters and a
  return type, which can be `T?`, `T ! E`, `T ! A | B`, `!T`, `!` or `! E`;
  `struct` with `name Type` fields; `enum` with variants that may carry
  fields; `error struct` and `error enum`; methods `fn Type.name(self, ...)`
  and `fn Type.name(mut self, ...)`; associated functions `fn Type.name(...)`;
  type parameters with constraints on functions, structs, enums and methods
  (`fn f[T Ord](...)`, `struct Stack[T]`, `fn Stack[T Ord].max(self)`,
  `fn Pair.with[C](...)`); `trait` with required and default methods;
  `impl Trait for Type`, including empty `impl`s that derive `Eq`, `Ord` or
  `ToStr`, and `impl`s for `int`, `str`, `bool` and `List`
- **Statements:** `x := v`, `mut x := v`, `x := v else fallback`,
  `x := v else return ...`, `x := v else` or `else err` with a block,
  `p = v`, `p op= v` (for `+ - * / %`), `if`/`elif`/`else`, including
  binding conditions `if x := v` and `else err`, `while`,
  `for x in list`, `for mut x in list`, `for i in a..b` and `a..=b`,
  `break`, `continue`, `return`
- **Expressions:** `if` and `match` as values; `match` on enums with variant,
  `Variant(a, _)` and `_` patterns; `e?` and `e? "context"`; `none`; calls,
  with type arguments inferred or written (`f[int](x)`, `Stack[int].new()`);
  trait method calls, including qualified ones (`Shape.area(x)`);
  `==` and `!=` on any type that supports them, and `<` on `Ord` types;
  `mut` arguments, method calls, fields, indexing `xs[i]`, slices `xs[a..b]`;
  struct and variant literals `Type(field: value)`; list literals; operators
  by ADR 0112's precedence; string interpolation `"{expr}"`; bare enum
  variants where the type is known
- **Types:** `int` (`i64`), `bool`, `str`, `List[T]`, `T?`, structs, enums,
  generic instances such as `Stack[int]`, type parameters, `Self`,
  `any Trait`, and, as return types only, `T ! E` and `!T` with inferred
  error sets
- **Library:** `print` (any value), `xs.len()`, `xs.push(v)`,
  `xs.swap(i, j)`, `s.split(sep)`, `s.to_int()`, and the standard traits'
  `x.eq(y)`, `x.cmp(y)` (returning `Ordering`) and `x.to_str()`
- **Semantics:** value semantics with copy-on-write; `mut` operands are moved
  into calls and written back; a read-only argument that overlaps a `mut` one
  is a copy; overflow, division, index and overlapping-`mut` traps; failures
  carry context and a return trace, and `main` reports them; generic code
  runs on the values it gets, and trait methods dispatch on the value's
  type, so `any Trait` needs no boxing

When `main` fails, the report on standard error is the error in its default
form (`err.text.1`), each context message, and each location in the return
trace, innermost first:

```
error: ConfigError.Missing(key: "host")
  context: loading the host
  at config.ail:7:5
  at config.ail:11:27
```

Not yet: floats, `char`, `T ! E` outside return types, matching on a set of
error types, literal patterns, maps and sets, the arithmetic operator traits,
trait functions without `self` and supertraits (OQ-039), associated types
(OQ-037), methods for one instantiation only, modules, `defer`, `ref`,
`unique`, tasks, `unsafe` and FFI.

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
     print `NAME.out` and then trap with the rule ID in `NAME.trap`;
   - `NAME.ail` + `NAME.err` (+ optional `NAME.out`) for programs whose `main`
     must fail with exactly the report in `NAME.err`, with `$FILE` standing
     for the program's path.
4. Keep the code formatted with `rustfmt` (settings in `rustfmt.toml`) and free
   of `clippy` warnings. Neither is in the minimal toolchain the designer
   uses, so agents add them locally (`rustup component add rustfmt clippy`).
5. `./ail selftest` must pass on Linux, and `tools/verify.sh BRANCH` on macOS
   (ADR 0085).
