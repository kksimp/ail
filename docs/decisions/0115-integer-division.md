# 0115. Integer division truncates toward zero

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 2 — Core syntax (expressions)
- **Spec rules:** [expr.arith.5](../spec/05-expressions.md)

## Context

`expr.arith.3` said when `/` and `%` trap, but not what they compute for
negative operands.

## Deciding criterion

Token usage **ties**. **AI understanding** favours truncation: it's what C,
C++, Rust, Go, Java, C#, Swift, Zig and JavaScript do, so most systems code an
AI has read assumes it. **Speed** agrees: truncation is a single instruction
on both ISAs (`idiv` on x86-64, `sdiv` on ARM64), while floor division
needs extra correction instructions on every signed division.

## Decision

For integers, `a / b` is the quotient rounded toward zero, and `a % b` has
the sign of `a`, so that `a == (a / b) * b + a % b`. For example,
`-7 / 2 == -3` and `-7 % 2 == -1`.

## Consequences

- `a % -1` is `0` for every `a`, including the minimum value: the result
  is representable, so unlike `a / -1` it doesn't trap (`expr.arith.3`).
  On x86-64 the compiler adds the same check it needs for division.

- Floor and Euclidean division and remainder (useful for wrap-around indexes)
  will be library functions.

## Rejected alternatives

- **Floor division** (Python, Ruby): nicer for negative modular arithmetic,
  but slower and different from almost every systems language.
- **Euclidean** (`%` always non-negative): same costs as floor division.
