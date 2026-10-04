# 05. Expressions and statements

Chapter key: `expr`

> **Status:** Drafted (partial). Only integer arithmetic semantics are
> specified.

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
