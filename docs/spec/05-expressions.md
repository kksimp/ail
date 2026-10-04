# 05. Expressions and statements

Chapter key: `expr`

> **Status:** Drafted (partial). Bindings, the `match` arm form, and integer
> arithmetic are specified.

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

> **Note:** the `for` syntax in the example is illustrative and not yet decided.

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

> **Note:** pattern syntax, guards, and warning-suppression syntax are not yet
> decided.

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

> **Note:** "trap" is defined in the [glossary](../glossary.md). What a trap
> does (whether it stops the task or the program, and how it is reported) is
> decided in the error-handling chapter.
