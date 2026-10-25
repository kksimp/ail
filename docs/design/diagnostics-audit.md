# Diagnostics audit: what an AI gets wrong, and what AIL says

> 2026-10-20, lane 45. Non-normative. Rules: `gram.foreign.1`, `gram.rec.1–3`,
> `gram.help.1–2` in [03-grammar.md](../spec/03-grammar.md); decisions
> [0262](../decisions/0262-report-foreign-syntax-once-with-help.md),
> [0263](../decisions/0263-one-diagnostic-per-mistake-recovery.md),
> [0264](../decisions/0264-anchored-call-rewrite-fixes.md).

AIL is read and written by AI agents, and an agent learns the language from the
first diagnostic it sees. The corpus in `tests/mistakes/` (109 small
programs) holds one plausible mistake each, written as if AIL were Python,
Rust, Go or TypeScript. Each has `NAME.rules` (the diagnostics' rule IDs) and,
when `ail fix` leaves it clean, `NAME.fixed`. the gate's mistakes suite
runs it; it fails if a diagnostic loses its help, a cascade returns, or fewer
than 60% of the programs are fixed.

## Results

Measured with the binary before this lane (`cf198f4`) and after, on the same 109 programs:

| | Before | After |
|---|---|---|
| Programs with a cascade (more than one report for one slip) | 9 | 2 (each holds two separate slips) |
| Diagnostics reported | 120 | 111 |
| First diagnostic has a `help` | 55 (50%) | 109 (100%) |
| Reported as `bootstrap.unsupported` (a mistake, not a missing feature) | 3 | 0 |
| Resolved by `ail fix` (checks clean afterwards) | 10 (9%) | 74 (68%) |

Target was 60% fixed. Speed and tokens: no change to the interpreter, and the
language card is untouched. `tools/perfbench` shows no difference beyond noise (see the PR).

## What changed

- **Foreign syntax is named once** (`gram.foreign.1`): `def`, `class`, `import`,
  `async`, `try`, `switch`, `pass`, `new`, `x++`, `<>`, `::`, `Point{...}`,
  braces, `:` after a header, `//`, `\`, `f"..."`, and calls such as `len(xs)`,
  `str(n)`, `range(n)`, `println(x)`. The tables are in `bootstrap/src/foreign.rs`.
- **One report per cause** (`gram.rec.1–3`): a whole 2-space or tab-indented
  file is one diagnostic with a re-indenting fix; an unclosed bracket doesn't
  cascade; `x = 1` doesn't make every later `x` an error; `p.y` no longer
  "fixes" to `p.x`.
- **Every error has a help** (`gram.help.1`): a default per rule when the site has none.
- **Fixes that rewrite an expression** (ADR 0264): call rewrites, `Option[T]`
  to `T?`, naming positional fields, `0 < n < 5`, `if n` to `if n != 0`.

## Not fixable, by design

The 35 programs `ail fix` leaves are those where the right edit needs a
decision: no exceptions (`try`, `throw`), `switch`, `a ? b : c`, `in`,
`for i, x in`, lambdas, wrong argument counts and types, a missing `main`, a
missing `else` value, an unterminated string, string `+` (operand extents
aren't in the tree), `int(x)` and `input()` (the right method depends on the
type). Each still has a help that says what to write.

## Table

First diagnostic of each program (help truncated at 90 characters).

| Mistake | Rule | Help | `ail fix` |
|---|---|---|---|
| `abs_function` | `expr.name.1` | `abs` is a method: write `n.abs()` | yes |
| `and_and` | `expr.op.1` | write `and` | yes |
| `arrow_return` | `gram.fn.2` | remove the `->` | yes |
| `assign_in_condition` | `expr.bind.3` | compare with `==` | yes |
| `assign_to_immutable_field` | `expr.bind.3` | declare it with `mut p := ...` | yes |
| `async_fn` | `gram.foreign.1` | tasks are colorless: start any function with `spawn f(x)` and wait with `t.wait()` | yes |
| `bang_not` | `expr.op.1` | write `not` | yes |
| `brace_escape` | `lex.str.2` | write `\{` for a literal `{`, or use a raw string `r"..."` | yes |
| `braces` | `gram.foreign.1` | a block is the lines indented 4 spaces below its header, with no closing line | yes |
| `chained_cmp` | `expr.prec.2` | combine them with `and`, e.g. `a < b and b < c` | yes |
| `class_keyword` | `gram.foreign.1` | declare a `struct` with its fields, then methods as `fn Name.method(self)` | yes |
| `colon_return` | `gram.fn.2` | remove the `:` | yes |
| `comment_slashes` | `gram.foreign.1` | comments start with `#` | yes |
| `const_keyword` | `mod.const.1` | remove the keyword and write `NAME := value` | yes |
| `def_keyword` | `gram.foreign.1` | write `fn name(a int) int` | yes |
| `dict_literal_colon_type` | `expr.bind.3` | declare it with `mut m := ...` | yes |
| `else_colon` | `gram.foreign.1` | a line that opens a block ends where the header ends; the block is the indented lines b... | yes |
| `else_if` | `expr.if.1` | write `elif` | yes |
| `elseif_one_word` | `gram.foreign.1` | write `elif` | yes |
| `enum_double_colon` | `gram.foreign.1` | write `Color.Red`, `std.fs` | yes |
| `eq_declare` | `expr.bind.3` | declare it with `mut x := ...` | yes |
| `eq_none_python` | `expr.name.1` | the absent value is `none`, and the type must say it can be absent: `int?` | yes |
| `field_colon` | `type.struct.2` | write `x <Type>` | yes |
| `for_c_style` | `expr.loop.1` | loop over a range: `for i in 0..n` (excludes `n`) or `for i in 1..=n` (includes it); us... | no |
| `for_enumerate` | `expr.loop.1` | for the index and the item, loop over the indexes: `for i in 0..xs.len()` then `xs[i]`;... | no |
| `for_in_range_fn` | `expr.name.1` | a range is written `0..n` (excludes `n`) or `1..=n` (includes it) | yes |
| `for_map` | `type.map.5` | write `for k in m.keys()`, or `m.values()` | yes |
| `from_import` | `gram.foreign.1` | import the module with `use std` and write `std.name` for its items | yes |
| `fstring` | `gram.foreign.1` | every string interpolates `{expression}`; write the string without the prefix | yes |
| `generic_angle` | `gram.foreign.1` | write `List[int]()` | yes |
| `if_int` | `expr.if.2` | compare explicitly, e.g. `n != 0` | yes |
| `immutable_list_push` | `trait.method.3` | declare it with `mut xs := ...` | yes |
| `impl_without_trait` | `trait.impl.1` | methods of a struct are plain functions: `fn Point.name(self) T` | no |
| `import_keyword` | `gram.foreign.1` | write `use std.fs` | yes |
| `in_operator` | `gram.foreign.1` | write the test as a method on the collection: `xs.contains(x)`, `m.contains(key)`, `s.c... | no |
| `input_function` | `expr.name.1` | read input with `use std.io` and `io.read_line()`, which gives `str?` (`none` at end of... | no |
| `int_function` | `expr.name.1` | convert text with `s.to_int()`, which gives `int?` (`none` if it isn't a number), and a... | no |
| `is_none` | `gram.foreign.1` | compare with `==`, e.g. `x == none` | yes |
| `lambda_arrow` | `gram.foreign.1` | declare a named `fn` and pass its name | no |
| `len_function` | `expr.name.1` | `len` is a method: write `xs.len()` | yes |
| `len_no_parens` | `expr.op.3` | write `.len()` | yes |
| `length_method` | `expr.name.1` | write `.len()` | yes |
| `length_property` | `type.struct.6` | `length` isn't a field; write the method `.len()` | yes |
| `let_binding` | `expr.bind.1` | write `name := value`, or `mut name := value` to allow reassignment | yes |
| `list_append` | `expr.name.1` | write `.push()` | yes |
| `list_typed_decl` | `expr.bind.1` | write `xs := value`; the type is inferred. An empty collection names its type in the va... | no |
| `match_colon_arm` | `expr.match.1` | write `=>` | yes |
| `match_not_exhaustive` | `expr.match.2` | add an arm for each, or a `_` arm | yes |
| `max_function` | `expr.name.1` | `max` is a method: write `a.max(b)` | yes |
| `method_in_struct` | `type.struct.2` | declare the method after the struct, with the type's name: `fn Name.method(self) T` | no |
| `missing_else_value` | `expr.if.4` | add an `else` branch | no |
| `missing_field` | `type.struct.4` | every field must be given; there are no defaults yet; `todo` stands in for a value you ... | yes |
| `missing_main` | `mod.main.1` | add:     fn main()         print("Hello, world!") | no |
| `missing_mut_call` | `own.param.2` | write `mut` before the argument | yes |
| `missing_return_type` | `mod.main.3` | if `add` should give a value, declare its type after the `)`: `fn add(...) int` | no |
| `multichar_single_quotes` | `lex.str.4` | write a string in double quotes: "..." | yes |
| `mut_not_declared` | `expr.bind.3` | declare it with `mut n := ...` | yes |
| `negative_index` | `type.coll.4` | count from the end with `xs.len()`: `xs[xs.len() - 1]`, or `xs.last()` | yes |
| `new_keyword` | `gram.foreign.1` | construct a struct by calling its name: `Point(x: 1, y: 2)` | yes |
| `new_line_continuation_backslash` | `gram.foreign.1` | end the line with an operator or a comma, or wrap the expression in parentheses | yes |
| `none_non_optional` | `type.opt.4` | use `none` where a `T?` is expected, e.g. as an argument or a return value | no |
| `not_equal_diamond` | `gram.foreign.1` | write `!=` | yes |
| `null_literal` | `expr.name.1` | the absent value is `none`, and the type must say it can be absent: `int?` | no |
| `option_type` | `type.opt.3` | write `int?` for `Option[int]`; the two mean the same and `T?` is the canonical form | yes |
| `or_or` | `expr.op.1` | write `or` | yes |
| `param_colon` | `gram.fn.2` | write `a <Type>` | yes |
| `pass_statement` | `gram.foreign.1` | a block needs a statement; write `todo` for code you haven't written yet | no |
| `percent_format` | `trait.op.3` | AIL has no `%` formatting; interpolate the values: "n is {n}" | no |
| `plusplus` | `gram.foreign.1` | write `n += 1` | yes |
| `print_two_args` | `mod.main.3` | print one string that interpolates the values: `print("{a} {b}")` | yes |
| `printf_function` | `expr.name.1` | build the text with interpolation, `print("n is {n}")`; a width or precision goes after... | no |
| `println_fn` | `expr.name.1` | `print(value)` writes a value and a newline | yes |
| `python_elif_colon` | `gram.foreign.1` | a line that opens a block ends where the header ends; the block is the indented lines b... | yes |
| `python_slice` | `type.coll.4` | write `xs[a..b]`, `xs[a..]` or `xs[..b]` | yes |
| `return_missing_value` | `expr.block.2` | make the last expression the declared return type, change the return type, or add `retu... | no |
| `return_outside_value` | `expr.block.2` | remove the value, or declare a return type after `)` | no |
| `return_type_mismatch` | `expr.block.2` | make the last expression the declared return type, change the return type, or add `retu... | no |
| `self_missing_method` | `expr.name.1` | this function has no `self`, so the field `x` isn't in scope; add `self` as its first p... | yes |
| `semicolon` | `lex.stmt.3` | remove the `;` and put the next statement on its own line | yes |
| `shadow_walrus` | `expr.bind.4` | `n` is a `mut` variable; to change it, write `n = ...` | yes |
| `str_function` | `expr.name.1` | interpolate to get text: `"{n}"`, or write `n.to_str()` | yes |
| `string_index` | `type.str.2` | index its characters: `s.chars()[i]`, or `s.chars()[a..b].join("")` for a part | no |
| `string_plus` | `trait.op.1` | build text with interpolation: "{a}{b}" | no |
| `string_times` | `trait.op.3` | repeat a string with `s.repeat(n)` | no |
| `struct_literal_braces` | `gram.foreign.1` | write `Point(x: 1, y: 2)`: the name, then `field: value` pairs in parentheses | yes |
| `struct_positional` | `type.struct.7` | name them: `Point(x: ..., y: ...)` | yes |
| `switch_keyword` | `gram.foreign.1` | write `match value` with one `pattern => result` arm per line | no |
| `tab_indent` | `lex.block.4` | indent with 4 spaces per level | yes |
| `ternary` | `gram.foreign.1` | use an `if` expression: `x := if n > 0` with the value indented below, then `else` and ... | no |
| `throw_keyword` | `gram.foreign.1` | errors are values: declare `fn f() T ! MyError`, fail with `return Variant`, and handle... | no |
| `true_python` | `expr.name.1` | booleans are `true` and `false` | yes |
| `try_keyword` | `gram.foreign.1` | errors are values: declare `fn f() T ! MyError`, fail with `return Variant`, and handle... | no |
| `two_space_indent` | `lex.block.2` | use 4 spaces per level | yes |
| `typed_binding` | `expr.bind.1` | write `x := value`; the type is inferred. An empty collection names its type in the val... | yes |
| `typo_name` | `expr.name.1` | did you mean `count`? | yes |
| `unclosed_paren` | `lex.stmt.2` | close the bracket on the same statement, or remove it | yes |
| `unclosed_string` | `lex.str.1` | close the string with `"` | no |
| `undefined_name` | `expr.name.1` | no variable, function or type named `y` is in scope here; declare it first, e.g. `y := ... | no |
| `unhandled_result` | `err.use.1` | pass the error on with `?`, or handle it with `x := ... else ...` | no |
| `unknown_field` | `type.struct.6` | `Point` has the fields `x` | no |
| `unknown_type` | `expr.name.1` | write `int` | yes |
| `unreachable_dedent` | `lex.block.2` | use 4 spaces | no |
| `unwrap_method` | `expr.name.1` | a value that can be `none` has no methods yet: get the value first, `v := x else defaul... | no |
| `use_rust_path` | `gram.foreign.1` | write `use std.fs` | yes |
| `var_binding` | `expr.bind.1` | write `name := value`, or `mut name := value` to allow reassignment | yes |
| `while_paren_colon` | `gram.foreign.1` | a line that opens a block ends where the header ends; the block is the indented lines b... | yes |
| `while_true_python` | `gram.foreign.1` | a line that opens a block ends where the header ends; the block is the indented lines b... | yes |
| `wrong_arg_count` | `expr.call.1` | give exactly the arguments the signature lists; `ail api FILE` shows each signature | no |
| `wrong_arg_type` | `expr.call.2` | pass a value of the expected type: convert with `n.to_float()`, `x.to_int()`, or `"{x}"... | no |

## Lane 147: fixes for the bit operators (expr.bit.1)

`&` and `^` are reported as `gram.foreign.1` with a fix that rewrites
`l & r` as `l.bit_and(r)` and `l ^ r` as `l.bit_xor(r)` (`bootstrap/src/fix/bit.rs`,
`diag/rewrite.rs` `Anchor::Operands`). The rewrite applies only when both
operands are whole atoms of the line, so `a & 1 + 2` keeps its error: it
could change precedence. Corpus: 143 programs, 90 fixed (63%), up from 140
and 88. `|` and `~` have no fix yet: `|` is a closure delimiter in the lexer,
and `~` is reported as `bootstrap.unsupported`.

Dates (lane 147, PR #122's `std.time.date`): an unhandled `date.parse_date`
stays `err.use.1` with no fix. The signature fix (`! E`) is not offered for an
error type from another module, since its name is written with the module's
alias (`date.DateError`), so the fix would add an unresolved name. Corpus:
144 programs, 90 fixed (63%). The named-field fix for `date.Date(2026, 10, 20)`
(`type.struct.7`) doesn't apply to a qualified call yet, since its anchor
needs the callee as written.

Networking and regex (lane 147): an unhandled `net.get` or `regex.compile`
stays `err.use.1` with no fix, for the same reason as dates: the error type is
another module's. Corpus: 146 programs, 90 fixed (62%). `regex.search` (a
Python name) is `std.use.2` with no fix; the help lists the real functions.

Braces in strings (lane 147, from the whole-task study): `lex.str.2` for an
empty `{}` or an unclosed `{` now has a fix that writes `\{` (`bootstrap/src/fix/string.rs`).
A `}` after the escaped brace stays literal. Corpus: 148 programs, 92 fixed (62%).

Named arguments (lane 147, ADR 0454): a positional argument after a named one
is `expr.call.3` with a fix that names it with the parameter it fills
(`bootstrap/src/fix/named.rs`); a duplicate parameter gets a help and no fix,
since which of the two arguments is meant is a choice. Corpus: 150 programs,
93 fixed (62%).

Whole-number float defaults (lane 147): `n else 2.0` where `int` is expected
is `err.else.1` with a fix to `2` (`bootstrap/src/fix/number.rs`; the anchored edit
matches `2.0` only). Corpus: 151 programs, 94 fixed (62%). The whole-task t2 slip
is a different error: `doc.num` gives `float`, so `retries + 1` ends in `float`
under `!int`. Its fix is a choice (`.to_int()`, or `!float`), so none is applied.

Bit operators, the rest (lane 147): `a | b` in a call's argument list is
`gram.foreign.1` with a fix to `a.bit_or(b)` (both parsers; the AIL parser is
ported). `~a` is `gram.foreign.1` with help `a.bit_not()` and no fix, since the
operand's extent isn't in the tree for a prefix operator. Corpus: 153 programs,
95 fixed (62%). `date.Date(2026, 10, 20)` (`type.struct.7`) now gets the named-field fix:
the call anchor reads the dotted callee as written (`date.Date`).

Qualified constructors (lane 147): `type.struct.7` on `date.Date(2026, 10, 20)` now
has the named-field fix; the call anchor reads the dotted callee as written. Corpus:
154 programs, 96 fixed (62%).

## Round two (ADRs 0646-0648, lane 197)

Corpus: 250 programs (154 + 96 new), `ail fix` resolves 150 (60.0%; the
96 new ones: 54, 56%). Rules with a fix: 48 (was 41). Fix rules and the
decisions behind them are in [ADR 0648](../decisions/0648-round-two-fix-rules-and-the-corpus-target.md).

Diagnostic output on the 96 new programs, batch head vs this change
(`ail check --json`, error lines only): 33,997 bytes before, 36,143 after
(+6%, about 500 tokens at 4 bytes a token). Help lines: 257 before, 264 after.
The growth is the help texts that were empty or generic before (ADR 0647),
plus the new fix edits. Rule changes: `<<`/`>>` and tuples from `gram.syntax.1`
to `gram.foreign.1`; `case` arms and open or three-dot ranges moved to
`expr.match.1` and `expr.match.12`; `@` in a call to `gram.foreign.1`.

Still open: a silent `t[0][1]` on a one-dimensional tensor (OQ-122); the
program is left out of the corpus. A `(a, b)` tuple has no fix (needs the
closing parenthesis extent). `regex.sub` has no fix (argument order differs
from Python).
## Token audit of the mistakes corpus (lane 220, 2026-10-21)
Non-normative. The corpus has 154 programs (not 250), and they give 156
diagnostics. Each diagnostic is measured as the text `ail check FILE` prints for
it: the header, the source line and caret, `help`, `note`, `fix` lines
(`mistral-spm-v1` column of tokbench). Each program is run from its own
directory, so the path is the bare file name an agent would see. Run from the
repository root, the path `tests/mistakes/` adds about 1,480 tokens
(83.4 mean against 73.9); that is a corpus artifact, not a diagnostic cost.
| Measure | Before | After |
|---|---|---|
| diagnostics | 156 | 156 |
| total tokens | 11,534 | 11,456 |
| mean tokens per diagnostic | 73.9 | 73.4 |
Where the tokens are (before, all 156): headers 6,406 with the repository path
(about 4,900 with bare names), help lines 3,470, source and caret lines 1,322,
`fix` lines 877, `note` lines 275. The 30 longest are 88–177 tokens each (with
the repository path); most are one help sentence and one note.
What changed: the text form no longer repeats a `fix:` line whose title equals
the `help:` line (11 cases). JSON output (ADR 0145) keeps every fix title. Net
saving: 78 tokens, 0.7%.
What was not changed, and why:
- Help sentences: each one is the fix hint an agent acts on (`mut xs := ...`,
  the list of allowed field types, the `0..n` / `1..=n` forms). Shortening them
  removes the hint or its examples, which the lane's rule forbids.
- The `note:` location for derived code (`at gen:impl:...`): it is the only
  handle to the generated code, so it stays.
- Source and caret lines: they give the location; the caret needs them.
- The `bit_and` help (about 100 tokens, two diagnostics): its last clause is the
  reason `&` is absent. A shorter wording was considered and rejected because it
  drops that reason.
Conclusion: the diagnostics are already close to the floor for their content.
The realistic cut is about 1%. The larger cost is the file path in the header,
which an agent pays only when it runs the compiler from the repository root.

## Round two fixes (lane 216, ADR 0695)

`ail fix` resolves 151 of the 250 round-two mistakes (60.4%), up from 150.
Added: `gram.foreign.1` for `~name`, rewritten as `name.bit_not()` (ADR 0695).
The 70% target needs 175; the 99 unresolved mistakes have no mechanical,
safe rewrite, so each is left unfixed on purpose. The grouping:

| Rule | Count | Why no fix |
| `expr.name.1` | 13 | Each needs a judgement: `int("5")`, `input()`, `null`, `e.message()` (`to_str` exists, but the target is the caller's intent), undefined names, `thread.start`, `m.push` on a non-list map |
| `gram.foreign.1` | 9 | Structural rewrites: ternary, `switch`, `throw`/`try`, lambdas, `match (a, b)`, tuple shapes, Python imports |
| `expr.call.1` | 6 | Arity or positional arguments: the right arguments are the caller's choice |
| `conc.task.5` | 5 | `t.wait()?` needs an `!` return type the mistake may not intend |
| `trait.op.3`, `trait.op.1` | 5 | Interpolation or conversion changes the result type |
| `expr.block.2` | 4 | Return type or value: which one the author meant |
| `err.use.1` | 4 | `else` or `?` needs a fallback value or a `!` return type |
| `conc.par.5` | 4 | Restructuring a `par` loop changes the program |
| `expr.call.2`, `expr.call.3` | 5 | Conversions or a choice of which duplicate to drop |
| `type.struct.6/7`, `type.opt.4`, `std.use.2`, `mod.use.2/4`, `type.bytes.2`, `type.fn.2`, `gen.*`, `contract.*`, `own.*`, `eff.pure.1`, `lex.*`, `expr.if/loop/match.*`, `json.derive.1`, `err.prop.2`, `conc.atomic/share.*`, `gram.fn.2`, `expr.key.1`, `std.use.4`, `mod.main.*`, `trait.impl.1`, `type.struct.2`, `type.bytes.2`, `set` | 44 | Each is one or two files; each has a choice a rewrite would guess (for example `missing_main`'s template, `method_in_struct`'s placement, `for_enumerate`'s loop shape, `unclosed_string`'s closing line) |

The obvious next candidates, if the designer accepts them as rewrites: a
`for i, x in xs` to indexed loop (`expr.loop.1`), positional-to-named struct
arguments (`type.struct.7`, the field names are in the declaration), and
`(a, b)` to `[a, b]` in `tensor` shape arguments (`gram.foreign.1`).
