# 05. Expressions and statements

Chapter key: `expr`

> **Status:** Drafted (partial). Specified: bindings, names, blocks and
> divergence, `if`, loops, ranges, logical and compound operators, comparisons,
> `match` (variant, literal and or-patterns), and integer arithmetic. Not yet
> designed: range patterns, guards, and binding or-patterns (see the note at
> the end of the match section).

## Bindings

**expr.bind.1** — `name := expr` declares an immutable local binding. An
immutable binding MUST NOT be reassigned. *(ADR 0038, ADR 0039)*

**expr.bind.2** — `mut name := expr` declares a mutable local binding.
*(ADR 0038, ADR 0039)*

**expr.bind.3** — `name = expr` assigns to an existing mutable binding. Using
`=` with an undeclared name, or with an immutable binding, is a compile-time
error. *(ADR 0039)*

**expr.bind.4** — Declaring a local name that is already visible from the same
or an enclosing local scope of the same function is a compile-time error. This
applies to variables, parameters, and loop and pattern bindings. *(ADR 0039,
ADR 0044)*

```ail skip=context
fn total(items List[Item]) i64
    mut sum := 0
    for item in items
        sum = sum + item.price
    sum
```

**expr.bind.5** — Declaring a local name equal to a visible module-level
item, imported name, or prelude name is a compile-time error. *(ADR 0084)*

**expr.bind.6** — `p = e` and `p op= e` require `p` to be a mutable place
(`own.place.1`), and `e` MUST have the type of `p`. *(ADR 0117)*

## Names

**expr.name.1** — Every name used in an expression MUST refer to a declaration
visible at that point. Otherwise it's a compile-time error. The diagnostic
SHOULD suggest the closest visible names. *(ADR 0107)*

**expr.name.2** — When an unknown function or method is the name another
mainstream language uses for something AIL spells differently, the
diagnostic MUST also give a fixed help that names the AIL form. The
implementation keeps these helps in a table. The table MUST contain at least
the entries below. A declared name is never overridden by the table.
*(ADR 0140, ADR 0107)*

| Habit | Help names |
|---|---|
| `len(x)`, `x.length`, `x.size`, `x.count` | `x.len()`; for a `str`, `s.chars().len()` |
| `str(x)` | `"{x}"` |
| `int(s)` | `s.to_int()` |
| `float(x)` | `s.to_float()` for text, `n.to_float()` for an `int` |
| `ord(c)`, `chr(n)` | `c.code()`, `char.from_code(n)` |
| `abs(x)` | `x.abs()` |
| `sorted(xs)` | `xs.sorted()` |
| `sum(xs)` | `xs.sum()` |
| `min(a, b)`, `max(a, b)`, `min(xs)`, `max(xs)` | `a.min(b)`, `a.max(b)`, `xs.min()`, `xs.max()` |
| `"".join(xs)`, `str.from_chars(xs)` | `xs.join("")` |
| `strip`, `lstrip`, `rstrip` | `trim` |
| `startswith`, `endswith` | `starts_with`, `ends_with` |
| `removeprefix`, `removesuffix` | `strip_prefix`, `strip_suffix` |
| `rjust`, `ljust`, `zfill`, `padStart` | `pad_left`, `pad_right`, with a fill char: `s.pad_left(4, '0')` |
| `splitlines` | `lines` |
| `isdigit`, `is_numeric`; `isalpha`, `is_alphabetic`; `is_whitespace` | the `char` methods `is_digit`; `is_alpha`; `is_space` |
| `contains_key`, `has`, `hasKey` | `contains` |
| `m.get(k, d)` | `v := m.get(k) else d` |
| `items()` | iterate `m.keys()` and index with `m[k]` |
| `append`, `add` on a list | `push` |
| `panic`, `assert` | `unreachable(message)` |
| `loop` | `while true` |

## Calls

**expr.call.1** — A call MUST supply exactly one argument per parameter of the
called function, in order. *(ADR 0108)*

**expr.call.2** — Each argument's type MUST equal the corresponding parameter's
type, or satisfy its constraint for a generic parameter. No implicit conversion
applies, except optional wrapping (`type.opt.5`) and conversion to `any Trait`
(`trait.any.3`). *(ADR 0108, ADR 0129)*

## Blocks

**expr.block.1** — A block is an expression. Its value is the value of its
last expression, or no value if it ends in a statement. *(ADR 0091)*

**expr.block.2** — A function returns the value of its body block. `return e`
exits the function early with value `e`. *(ADR 0091)*

**expr.block.3** — A statement **diverges** when it is `return`, `break` or
`continue`; a call to `unreachable(message)` (`err.model.2`); a `while true`
(the literal `true`) whose body has no `break` for that loop (`expr.loop.7`);
or an `if`/`else` or `match` all of whose branches diverge. A block whose last
statement diverges produces no value and is accepted wherever a value of any
type is expected, including as a one-line arm (`expr.match.9`) and as an
`else` part. So a function can end with `while true`. There is no nameable
`Never` type. *(ADR 0117, ADR 0139)*

## Conditionals and loops

**expr.if.1** — `if c` is followed by an indented block, then optionally any
number of `elif c` blocks and one `else` block. An `if` is an expression when
it has an `else` and every branch produces a value of the same type.
*(ADR 0091)*

**expr.loop.1** — `for x in e` runs its block once per element of `e`.
`while c` runs its block while `c` is true. *(ADR 0091)*

**expr.loop.2** — `break` exits the innermost loop. `continue` starts its
next iteration. *(ADR 0091)*

**expr.loop.3** — `a..b` is the half-open range from `a` up to but excluding
`b`. `a..=b` includes `b`. *(ADR 0091)*

**expr.loop.4** — In `for x in e`, `e` MUST be a `List[T]`, giving `x` type
`T`, a `Set[T]`, giving `x` type `T`, or a range of `int`, giving `x` type
`int`. A set is iterated in insertion order (`type.map.1`). `x` is an
immutable binding scoped to the loop body. The loop iterates the value `e` had
when the loop started. `e` MUST NOT be a `Map`: `for k in m` is a
compile-time error, and its help says to iterate `m.keys()` (the habit is
ambiguous between keys and pairs). *(ADR 0117, ADR 0131)*

**expr.loop.5** — `break` and `continue` outside a loop are compile-time
errors. *(ADR 0117)*

**expr.loop.6** — In `for mut x in e`, `e` MUST be a mutable place
(`own.place.1`) of type `List[T]`. `x` is a mutable place of type `T` that
refers to each element in turn, in order, so changes to `x` change the
element. The body's use of `e` is restricted by `own.excl.5`. `for mut` over
a `Set`, over `m.keys()` or over `m.values()` is a compile-time error: they
are not places. If `e` is a map entry `m[k]` (`own.place.1`), `k` MUST be
present, or the loop traps (`type.map.3`). *(ADR 0121, ADR 0131)*

**expr.loop.7** — A `while` whose condition is the literal `true` and whose
body contains no `break` for that loop never completes normally, so as a
statement it diverges (`expr.block.3`). A `break` inside a nested loop belongs
to that loop. AIL has no `loop` keyword: the help for it says `while true`.
*(ADR 0139)*

**expr.if.2** — Conditions of `if`, `elif` and `while` MUST have type `bool`,
apart from binding conditions (`expr.if.5`). *(ADR 0110, ADR 0125)*

**expr.if.3** — When an `if` begins inside a statement (e.g. as a binding's
value), its blocks are indented one level deeper than that statement's line,
and its `elif` and `else` lines are at that statement's level. *(ADR 0111)*

```ail skip=context
kind := if n < 0
    "negative"
elif n == 0
    "zero"
else
    "positive"
```

**expr.if.5** — A condition of `if` or `elif` may be a binding `name := e`,
where `e` has type `T?` or `T ! E`. When `e` is present or succeeded, `name`
is bound to its value, immutably and only within that branch, and the branch
runs. Otherwise the next `elif` or `else` is tried. *(ADR 0125)*

**expr.if.6** — When an `if` has exactly one condition, and it binds a value
of type `T ! E`, its `else` may be written `else name`, which binds the error
value to `name` within the `else` block. The type of `name` is given by
`err.else.3`. *(ADR 0125, ADR 0139)*

```ail skip=context
if cfg := parse(lines)
    print("ok: {cfg.width}x{cfg.height}")
else err
    print("error: {err}")
```

**expr.if.4** — An `if` or `match` whose value is used (bound, passed,
returned, or ending a block whose value is used) MUST produce a value on every
path, so such an `if` MUST have an `else`. All its branches or arms MUST have
the same type. *(ADR 0117)*

## Operators

**expr.prec.1** — Operators bind, from tightest to loosest: postfix (call,
field, method, index, `?`); unary `-`; `* / %`; `+ -`; comparisons
`== != < <= > >=`; `not`; `and`; `or`; ranges `.. ..=`; `else`
(`err.else.6`). Binary operators at the same level associate to the left,
except comparisons, ranges, and `else`, which associates to the right.
*(ADR 0112, ADR 0144)*

**expr.prec.2** — Two comparison operators in sequence without parentheses
(e.g. `a < b < c`) are a compile-time error. *(ADR 0112)*

**expr.op.1** — The logical operators are `and` and `or` (both
short-circuiting) and `not`. *(ADR 0092)*

**expr.op.2** — `x op= e` for `op` in `+ - * / %` means `x = x op e`. `x`
MUST be a mutable place. *(ADR 0092)*

**expr.op.3** — Every method call is written with parentheses, including
calls with no arguments. *(ADR 0092)*

**expr.cmp.1** — In `a == b`, `a != b`, `a < b`, `a <= b`, `a > b` and
`a >= b`, both operands have the same type, and the expected type of `b` is
the type of `a` (so `d == East` needs no qualification, `type.enum.3`).
`==` and `!=` need a type implementing `Eq`; the others need `Ord`
(`trait.std.1–2`). On `float` operands all six are the IEEE 754 comparisons
(`type.num.6`): NaN is unequal to everything, itself included, and every
ordering comparison with NaN is false. *(ADR 0129, ADR 0127, ADR 0130)*

**expr.cmp.3** — `==` or `!=` between an optional and a negative integer
literal is a compile-time error. The help says that `index_of` returns `none`
when the element is absent, not `-1`. *(ADR 0136)*

```ail skip=context
i := xs.index_of(x)
if i == -1        # compile-time error: index_of returns none when absent
    print("absent")
```

## Match

**expr.match.1** — A `match` expression is the keyword `match`, the
scrutinee expression, and an indented block containing one arm per line. Each
arm is written `pattern => result`. The result is an expression, or a
one-line statement on the arm's line (`expr.match.9`). A result consisting of
several statements is written as an indented block following `=>`.
*(ADR 0054, ADR 0138)*

```ail skip=context
area := match s
    Circle(r) => pi * r * r
    Rect(w, h) => w * h
    Empty => 0.0
```

**expr.match.2** — A `match` MUST be exhaustive. A non-exhaustive match is a
compile-time error, and the diagnostic MUST list the uncovered cases.
*(ADR 0056)*

**expr.match.3** — When the scrutinee has an enum type and a catch-all arm
`_` covers one or more of its named variants, the implementation MUST warn
and name those variants, unless the warning is suppressed for that match.
*(ADR 0056)*

**expr.match.4** — A pattern is `_`; a variant of the scrutinee's enum type,
bare or qualified; a literal (`expr.match.7`); or an or-pattern
(`expr.match.8`). A variant with data is followed by a parenthesised pattern
for each of its fields (`expr.match.6`). *(ADR 0117, ADR 0123, ADR 0138)*

**expr.match.5** — An arm whose pattern is already covered by earlier arms is
a compile-time error. Literals are compared by value (`0x41` after `65` is
redundant), and the rule applies to each alternative of an or-pattern.
*(ADR 0117, ADR 0138)*

**expr.match.6** — In a pattern `Variant(p1, ..., pn)`, there MUST be one
`pi` per field, in declaration order. Each `pi` is a new immutable binding
name or `_`, which ignores the field. *(ADR 0123)*

**expr.match.7** — A literal pattern is an `int` literal (with an optional
`-`), a `char` literal, a `str` literal without interpolation (a raw string is
allowed), or a `bool` literal. The literal MUST have the scrutinee's type, with
no conversion; when a `char` and a `str` are confused, the help offers the
other quote. A `float` literal pattern is a compile-time error, and its help
says to compare with `==` or `<` in an `if`. A match on `int`, `str` or `char`
MUST have a `_` arm even when every value is listed; a match on `bool` is
exhaustive with `true` and `false`. A `-` directly before an integer literal
is part of the literal for `lex.num.3`, so `-9223372036854775808` is valid in
patterns, expressions and constants. *(ADR 0138, ADR 0130)*

**expr.match.8** — `p1 | p2 | ...` is an or-pattern. It matches when any
alternative matches. No alternative may bind a name; a variant pattern whose
fields are all `_` is fine. *(ADR 0138)*

**expr.match.9** — The result of an arm may be a one-line statement on the
arm's line: `return x`, `break`, `continue`, `total += n`, `x = y` or
`print(x)`. A statement that diverges (`expr.block.3`) is accepted wherever
the match's type is expected. Any other statement gives the arm no value, so
the match can't be used as a value (`expr.if.4`). *(ADR 0138, ADR 0139)*

```ail skip=context
fn class(c char) int
    match c
        '0' | '1' => 1
        'a' | 'b' | 'c' => 2
        '\n' => return 0
        _ => 3

fn kind(t Token) str
    match t
        Number(_) | Label(_) => "atom"
        Plus => "op"
```

> **Note:** not yet designed: range patterns (`'0'..='9'`), guards
> (`x if x > 0`), or-patterns that bind names, patterns nested inside a
> variant's fields (`expr.match.6`), and warning-suppression syntax. The
> diagnostics for range patterns and guards suggest an `if` chain.
> *(ADR 0138)*

## Integer arithmetic

**expr.arith.1** — When the mathematical result of `+`, `-` (binary or
unary) or `*` on fixed-width integer operands can't be represented in the
result type, the operation MUST trap. It MUST NOT wrap silently, and it MUST
NOT have undefined behaviour. *(ADR 0035)*

**expr.arith.2** — The language MUST provide explicit wrapping, saturating
and checked forms of the arithmetic operations. A checked form returns a value
that indicates overflow instead of trapping. *(ADR 0035; syntax to be decided)*

**expr.arith.3** — Integer division or remainder by zero, and signed division
of the minimum value by `-1`, MUST trap. *(ADR 0035)*

**expr.arith.4** — An implementation MAY omit an overflow check that it proves
can never fail. It MUST report the overflow checks that remain in the
performance report. *(ADR 0035)*

**expr.arith.5** — Integer `a / b` rounds toward zero, and `a % b` has the
sign of `a`, so that `a == (a / b) * b + a % b`. `a % -1` is `0` for every
`a`, since that result is representable. *(ADR 0115)*

> **Note:** "trap" is defined in the [glossary](../glossary.md). What a trap
> does (whether it stops the task or the program, and how it is reported) is
> decided in the error-handling chapter.

## Holes

**expr.hole.1** — `todo` is a *typed hole*: an expression that stands for code
not yet written. It is accepted wherever an expression is. Its type is the type
the context expects; where no type is expected, its type is unknown and every
use of the value is accepted. Each hole MUST be reported as a warning, not an
error, whose message states the expected type (or that none is known) and lists
the local names in scope with their types, sorted by name. *(ADR 0147)*

```ail
fn area(w int, h int) int
    todo

fn main()
    print("started")
```

The checker warns "hole: expected `int`; in scope: `h int`, `w int`", and the
program still runs: `area` is never called.

**expr.hole.2** — Evaluating a hole MUST trap (`err.trap.1`). *(ADR 0147)*

> **Note:** a hole lets a program be written top-down: the checker says what
> each gap needs, and the program runs until it reaches one. The bootstrap
> doesn't infer an unknown hole type from later uses; a later lane may.
