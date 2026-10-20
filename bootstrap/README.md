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
./ail test [--json] FILE           # run the @test functions
./ail effects [--json] FILE        # each function's inferred effects
./ail run --deny fs,proc FILE      # refuse a program that reaches fs or proc
./ail selftest                     # the bootstrap's own tests
```

Exit status: `0` success, `1` the program failed or trapped, `2` the program
was rejected or the command was invalid. Warnings (such as `expr.match.3`) are
reported but don't stop a program from running.

## Pipeline

| File | Stage | Spec rules enforced |
|---|---|---|
| `src/lexer.rs` | Text to tokens, including INDENT/DEDENT and interpolated strings | `lex.block.1–4`, `lex.stmt.1–3`, `lex.kw.1–2`, `lex.comment.*`, `lex.str.1–5`, `lex.str.7`, `lex.num.1–3` |
| `src/prelude.ail` | The standard traits `Eq`, `Ord`, `ToStr` and `enum Ordering`, read before every program at line 0; then, only when the program names `Json`, `JsonMember`, `JsonError` or `parse_json` and declares none of them, the `std.json` module after its `## @module json` marker | `trait.std.1–3`, `json.type.1`, `json.err.1`, `json.parse.1`, `json.print.1`, `json.query.1`, `json.prelude.1` |
| `src/parser.rs` | Tokens to syntax tree | `gram.fn.1–2`, `gram.list.1`, `gram.generic.1`, `gram.syntax.1`, `gram.attr.1–2`, `gen.decl.1`, `gen.bound.1`, `gen.method.1–3`, `gen.infer.2`, `trait.decl.1–2`, `trait.impl.1`, `trait.any.1`, `type.struct.1–3`, `type.enum.1–2`, `type.opt.2`, `err.decl.1`, `err.type.1–2`, `err.type.5`, `err.else.1–3`, `expr.bind.1–3`, `expr.if.1`, `expr.if.3`, `expr.if.5–6`, `expr.loop.1–3`, `expr.loop.6`, `expr.match.1`, `expr.match.6`, `expr.prec.1–2`, `expr.op.1–3` |
| `src/check.rs` | Names, types, places, control flow | `mod.main.1`, `mod.main.3`, `mod.unit.1`, `expr.name.1`, `expr.bind.1–6`, `expr.block.2–3`, `expr.call.1–2`, `expr.if.2`, `expr.if.4`, `expr.loop.3–5`, `expr.match.2–5`, `expr.op.2–3`, `type.bool.1`, `type.coll.2–7`, `type.str.2–3`, `type.str.11`, `type.struct.3–6`, `type.enum.3`, `type.enum.5`, `trait.method.1–5`, `trait.op.1`, `trait.op.3`, `own.param.1–2`, `own.excl.1–2`, `own.excl.5`, `type.opt.4–5`, `type.enum.6`, `type.str.4–5`, `expr.loop.6`, `expr.match.6`, `err.decl.1–3`, `err.type.1–2`, `err.type.5–6`, `err.prop.1–3`, `err.use.1–2`, `err.else.1–3`, `err.ctx.2`, `err.text.1`, `expr.if.5–6`, `expr.cmp.1`, `gen.decl.1–2`, `gen.bound.1–4`, `gen.method.1–3`, `gen.infer.1–2`, `gen.anon.1`, `trait.decl.1–2`, `trait.impl.1–6`, `trait.self.1`, `trait.res.1`, `trait.any.1–5`, `trait.std.1–4`, `trait.derive.1`, `trait.auto.1–4` (traits and generics in `src/check/traits.rs`), `test.attr.1–3`, `test.assert.2` (in `src/check/testing.rs`) |
| `src/check/effects.rs` | Effect inference over the call graph, `@pure`, `--deny`, `ail effects` | `eff.infer.1–6`, `eff.pure.1`, `eff.cap.1–3` |
| `src/interp.rs` | Runs checked programs | `mod.main.2–3`, `own.val.1–2`, `own.excl.3–4`, `expr.arith.1`, `expr.arith.3`, `expr.arith.5`, `expr.loop.4`, `expr.loop.6`, `type.coll.4`, `type.str.4–5`, `type.str.11`, `err.prop.1`, `err.else.1–3`, `err.ctx.1–3`, `err.text.1`, `err.trap.1`, `err.trap.3`, `gen.impl.1`, `trait.any.1`, `trait.std.1–4`, `trait.derive.1`, `trait.auto.1–2`, `test.assert.1`, `test.run.1–4` (in `src/interp/testing.rs`) |
| `src/diag.rs` | Text and JSON diagnostics, errors and warnings | ADR 0002 (rule IDs), ADR 0024 |

All errors in a file are reported in one pass, sorted by position.

## Supported subset (milestone 6: Python parity)

- **Tests (ADRs 0167–0168):** `@test fn name()` on top-level functions
  (attributes `@test` and `@pure` only; arguments are skipped), run by
  `ail test FILE [--json]`; `assert(cond)` and `assert_eq(a, b)` trap with
  `test.assert.1`. `NAME.test` cases hold the expected `--json` output.

- **Declarations:** `fn` with `name Type` and `mut name Type` parameters and a
  return type, which can be `T?`, `T ! E`, `T ! A | B`, `!T`, `!` or `! E`;
  `struct` with `name Type` fields; `enum` with variants that may carry
  fields; `error struct` and `error enum`; methods `fn Type.name(self, ...)`
  and `fn Type.name(mut self, ...)`; associated functions `fn Type.name(...)`;
  type parameters with constraints on functions, structs, enums and methods
  (`fn f[T Ord](...)`, `struct Stack[T]`, `fn Stack[T Ord].max(self)`,
  `fn Pair.with[C](...)`); `trait` with required and default methods;
  `impl Trait for Type`, including empty `impl`s that derive `Eq`, `Ord` or
  `ToStr`, and `impl`s for `int`, `float`, `str`, `char`, `bool` and `List`;
  top-level constants `NAME := expr` (constant expressions)
- **Statements:** `x := v`, `mut x := v`, `x := v else fallback`,
  `x := v else return ...`, `x := v else` or `else err` with a block,
  `p = v`, `p op= v` (for `+ - * / %`), `if`/`elif`/`else`, including
  binding conditions `if x := v` and `else err`, `while`,
  `for x in list`, `for mut x in list`, `for i in a..b` and `a..=b`,
  `break`, `continue`, `return`; `f() else err` with a block, or
  `f() else return`, on a result with no success value
- **Expressions:** `if` and `match` as values; `match` with variant,
  `Variant(a, _)`, literal (`1`, `-1`, `'a'`, `"s"`, `true`), or-pattern
  (`A | B`) and `_` patterns, and one-line statement arms (`=> continue`);
  `e?` and `e? "context"`; `none`; `unreachable("why")`; calls,
  with type arguments inferred or written (`f[int](x)`, `Stack[int].new()`);
  trait method calls, including qualified ones (`Shape.area(x)`);
  `==` and `!=` on any type that supports them, and `<` on `Ord` types;
  `mut` arguments, method calls, fields, indexing `xs[i]` and `m[k]`, slices
  `xs[a..b]`, `xs[a..]`, `xs[..b]`; struct and variant literals
  `Type(field: value)`, with shorthand (`Pos(row, col)`) and positional
  arguments when no two fields filled share a type; list, map and set literals; `List[T]()`,
  `Map[K, V]()`, `Set[T]()`; float and char literals; multi-line,
  one-line `"""..."""` and raw strings; operators
  by ADR 0112's precedence; string interpolation `"{expr}"` with format specs `"{x:>8}"`, `"{r:.2}"`, `"{n:0>4x}"` (ADR 0173); bare enum
  variants where the type is known
- **Types:** `int` (`i64`), `float` (`f64`), `bool`, `str`, `char`,
  `List[T]`, `Map[K, V]`, `Set[T]`, `T?`, structs, enums,
  generic instances such as `Stack[int]`, type parameters, `Self`,
  `any Trait`, and, as return types only, `T ! E` and `!T` with inferred
  error sets
- **Library:** `print` (any value); the methods of `str`, `char`, `int`,
  `float`, `List`, `Map` and `Set` in spec chapter 4 (`type.str.*`,
  `type.char.*`, `type.num.*`, `type.coll.*`, `type.map.*`); the standard
  traits' `x.eq(y)`, `x.cmp(y)` (returning `Ordering`), `x.to_str()`,
  `a.min(b)` and `a.max(b)`; automatic `Hash` for map keys and set elements
- **Methods:** inside a method, the receiver's fields and methods by bare
  name (`pos += 1`, `peek()`); `[]` and `{}` declared with `:=` are typed by
  their first use (`out.push(x)`)
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

Effects (ADRs 0163–0165): every function's effect set is inferred and
printed by `ail effects`; `@pure` and `--deny` are checked at compile time.
Calls to the standard modules (`fs.*`, `env.*`, `io.*`, `proc.*`, `time.*`)
are recognised by name; `unsafe` and `spawn` aren't parsed yet, so nothing
has those effects.

Not yet: `f32` and the sized integer types, `s.bytes()`, `T ! E` outside
return types, matching on a set of error types, range patterns and guards,
the arithmetic operator traits, trait functions without `self` and
supertraits (OQ-039), associated types (OQ-037), methods for one
instantiation only, modules and `pub`, `defer`, `ref`, `unique`, tasks,
`unsafe` and FFI. Known gaps: a constant expression that would call a user
`impl` of `Eq`, `Ord` or `ToStr` (`mod.const.2`) isn't rejected yet, and a
constant whose evaluation fails traps before `main` rather than at check
time.

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
