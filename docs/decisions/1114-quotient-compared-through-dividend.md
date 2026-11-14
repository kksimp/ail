# 1114. A quotient that is only compared is compared through its dividend

- **Status:** Accepted
- **Date:** 2026-11-14
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-14 for a quick fix to byte- and output-heavy code before 0.1 if one exists
- **Topic:** 23 — Native backend
- **Spec rules:** [native.range.4](../spec/23-native.md)
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md), [ADR 1063](1063-function-wide-value-ranges.md)

## Context

`fasta` ran at 1.55 to 1.7 times `gcc -O2` and `clang -O2` on Linux and the
Mac. Timing probes on the Mac (2026-11-14, 64 M bases) put about 50 ms of the
280 ms gap in `join` and `print` and the rest in the loop that picks a base:

```
t := seed * 100 / 139968
if t < 30 ... elif t < 50 ... elif t < 70
```

The three branches are on random data, so each is mispredicted often, and a
mispredicted branch costs the time it took to compute its condition. AIL
computed `t` with a multiply-high division first; gcc and clang compare
`seed * 100` with `30 * 139968` and never divide.

## Decision

In `fold.ail` (`divcmp.ail`), a `div` by a constant `C >= 2` whose every use
is `<`, `<=`, `>` or `>=` against a constant `K` is replaced in those
comparisons by its dividend and the constant `K * C` (or `(K + 1) * C`), under
the bounds of native.range.4. The division then has no use and is removed.
It is exact for every `int` dividend, negative ones included, because division
rounds toward zero (expr.arith.5); the proof is in `divcmp.ail`.

A quotient with any other use (an index, an operand, a block argument) is left
alone, comparisons included: the range passes read those comparisons to bound
the quotient.

## Rejected alternatives

- **If-conversion (a branch-free select).** It would remove the mispredictions
  themselves and is the larger win, but it needs a new MIR operation, both code
  generators and a profitability rule. Too large for a release candidate; for
  0.1.1.
- **Rewriting every comparison of a quotient.** Sound, but a quotient that is
  also used keeps the division, so nothing is saved and bounds-check
  elimination could lose a fact.
- **`==` and `!=`.** They need two comparisons; not seen in measured code.
- **Unsigned magic division for a dividend proved non-negative.** A smaller
  gain (one instruction of the chain) that touches both code generators.

## Consequences

`tests/cases/native_range_quotient` counts each ordering over dividends on
both sides of zero and of every boundary, for bounds the rule takes and bounds
it leaves. MIR goldens that held such a division change.
