# 05. Expressions and statements

Chapter key: `expr`

> **Status:** Drafted (partial). Specified: bindings, blocks, `if`, loops,
> ranges, logical and compound operators, `match`, and integer arithmetic.

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

```ail
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

## Calls

**expr.call.1** — A call MUST supply exactly one argument per parameter of the
called function, in order. *(ADR 0108)*

**expr.call.2** — Each argument's type MUST equal the corresponding parameter's
type, or satisfy its constraint for a generic parameter. No implicit conversion
applies, except optional wrapping (`type.opt.5`). *(ADR 0108)*

## Blocks

**expr.block.1** — A block is an expression. Its value is the value of its
last expression, or no value if it ends in a statement. *(ADR 0091)*

**expr.block.2** — A function returns the value of its body block. `return e`
exits the function early with value `e`. *(ADR 0091)*

**expr.block.3** — A block whose last statement is `return`, `break` or
`continue` produces no value and is accepted wherever a value of any type is
expected. *(ADR 0117)*

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
`T`, or a range of `int`, giving `x` type `int`. `x` is an immutable binding
scoped to the loop body. The loop iterates the value `e` had when the loop
started. *(ADR 0117)*

**expr.loop.5** — `break` and `continue` outside a loop are compile-time
errors. *(ADR 0117)*

**expr.if.2** — Conditions of `if`, `elif` and `while` MUST have type `bool`.
*(ADR 0110)*

**expr.if.3** — When an `if` begins inside a statement (e.g. as a binding's
value), its blocks are indented one level deeper than that statement's line,
and its `elif` and `else` lines are at that statement's level. *(ADR 0111)*

```ail
kind := if n < 0
    "negative"
elif n == 0
    "zero"
else
    "positive"
```

**expr.if.4** — An `if` or `match` whose value is used (bound, passed,
returned, or ending a block whose value is used) MUST produce a value on every
path, so such an `if` MUST have an `else`. All its branches or arms MUST have
the same type. *(ADR 0117)*

## Operators

**expr.prec.1** — Operators bind, from tightest to loosest: postfix (call,
field, method, index, `?`); unary `-`; `* / %`; `+ -`; comparisons
`== != < <= > >=`; `not`; `and`; `or`; ranges `.. ..=`. Binary operators at
the same level associate to the left, except comparisons and ranges.
*(ADR 0112)*

**expr.prec.2** — Two comparison operators in sequence without parentheses
(e.g. `a < b < c`) are a compile-time error. *(ADR 0112)*

**expr.op.1** — The logical operators are `and` and `or` (both
short-circuiting) and `not`. *(ADR 0092)*

**expr.op.2** — `x op= e` for `op` in `+ - * / %` means `x = x op e`. `x`
MUST be a mutable place. *(ADR 0092)*

**expr.op.3** — Every method call is written with parentheses, including
calls with no arguments. *(ADR 0092)*

## Match

**expr.match.1** — A `match` expression is the keyword `match`, the
scrutinee expression, and an indented block containing one arm per line. Each
arm is written `pattern => result`. A result consisting of several statements
is written as an indented block following `=>`. *(ADR 0054)*

```ail
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

**expr.match.4** — Until general patterns are designed, a pattern is `_` or
a variant of the scrutinee's enum type, bare or qualified. *(ADR 0117)*

**expr.match.5** — An arm whose pattern is already covered by earlier arms is
a compile-time error. *(ADR 0117)*

> **Note:** general pattern syntax, guards, and warning-suppression syntax are
> not yet decided.

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
sign of `a`, so that `a == (a / b) * b + a % b`. *(ADR 0115)*

> **Note:** "trap" is defined in the [glossary](../glossary.md). What a trap
> does (whether it stops the task or the program, and how it is reported) is
> decided in the error-handling chapter.
