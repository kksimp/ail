# 0695. `ail fix` for `~name`: the bitwise-not method

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** gram.foreign.1, expr.bit.1 (ADR 0465, ADR 0202)
- **Builds on:** [ADR 0648](0648-round-two-fix-rules-and-the-corpus-target.md)

## Context

The round-two mistakes corpus (ADR 0646) has 250 programs, and `ail fix`
resolved 150 of them (60%). The 70% target of lane 216 needs more fixes, but
most unresolved mistakes have no single correct rewrite. A fix must be
mechanical and safe, or it is not added (ADR 0202).

`~a` (bitwise not, from C and Python) is the one unresolved case with an exact
rewrite that the help already names: `a.bit_not()` (expr.bit.1). It is in the
corpus as `bit_not.ail`.

## Decision

The lexer attaches a fix to `gram.foreign.1` for `~` when the operand is a
single name that is not followed by `.`, `(` or `[`: `~name` becomes
`name.bit_not()`. Any other operand (`~(a)`, `~a.b`, `~7`) keeps the help and
gets no fix, since the method would bind to a different expression.

## Measurement

Corpus resolution by `ail fix`: 150 → 151 of 250 (60% → 60.4%). The 70% target
is not reached. The remaining unresolved mistakes are listed with the reason
each has no safe fix in `docs/design/diagnostics-audit.md`.

## Rejected alternatives

- **Fix every `~` expression by wrapping it in parentheses.** `~(a + 1)` would
  become `(a + 1).bit_not()`, which is correct, but `~f(x)` could bind to
  `f(x).bit_not()` and change its meaning; the operand rule keeps the fix to
  one name, where the rewrite cannot change precedence.
- **Rewrite `~` to `0 - a - 1`.** Not a bitwise not in general, and it reads
  worse than the method the help names.
- **No fix; help only.** Works, but leaves a mechanical case for agents to
  retype; the method form is the one the language documents.
