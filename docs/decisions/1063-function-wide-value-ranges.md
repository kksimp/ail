# 1063. Function-wide value ranges decide checks and comparisons

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put every speed slice in 0.1.0 on 2026-11-12
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.range.1-3](../spec/23-native.md)
- **Builds on:** [ADR 0391](0391-native-backend-optimisations.md) (fold.ail), [ADR 0611](0611-overflow-checks-by-range-guards.md) (loop ranges), [ADR 0816](0816-native-codegen-entry-addressing-constants.md) (non-negative division), [ADR 1050](1050-dominated-checks-and-field-alias-classes.md) (dominated checks)

## Context

The slow-spots study of 2026-11-10 (slice F9) found that range reasoning
exists only inside counted loops (`ranges.ail`, `bce.ail`) and for one shape
in fold.ail (`i + 1` after `i < n`). Everywhere else every checked `+`, `-`,
`*`, `/`, `%` and shift keeps its check: hash arithmetic on bytes and masks
(lzr's `hash3`, five `jo` on values below 2^24), counters stepping by more
than 1, counting down to 0, `x % k` fed to further arithmetic, and
comparisons a mask already decides. Non-negative division as a shift (ADR
0816) is proved by a separate pass with its own rules.

## Options considered

### Option A — one interval analysis per function, consulted by fold.ail (chosen)

`compiler/mir/vrange.ail` gives each value an interval by a forward walk in
reverse postorder to a fixed point. Branch comparisons add facts that hold in
the blocks their edge dominates and on that edge's arguments. Loop
parameters join what every edge brings and widen to the end of `int` after
three growths. `fold_in` calls it between its constant folding and CSE.

- **Pros:** one place for range facts that every function gets, at lowering
  and again after inlining; general (no loop shape); small (one file);
  dominators shared with fold's other two passes, so fold computes them once
  instead of twice.
- **Cons:** intervals only, no relations between two values (`i < xs.len()`
  is still bce.ail's job); widening loses a loop value's bound unless a
  branch in the loop restores it.

### Option B — extend ranges.ail beyond counted loops

- **Pros:** one range engine.
- **Cons:** it is built around a guard that tests held values before a
  versioned loop; making it function-wide means rewriting it, and the
  versioning passes depend on its exact answers.

### Option C — relational analysis (difference bounds, ABCD)

- **Pros:** proves `i < len` and two-pointer bounds as well.
- **Cons:** much larger, slower to compile, and riskier before 0.1.0; bce.ail
  and forward.ail already cover the bounds-check cases measured.

## Decision

1. A checked `add`, `sub`, `mul`, `neg`, `div`, `rem`, `shl` or `shr` loses
   its check where the intervals of its operands rule its trap out
   (native.range.1). An `add` or `mul` so freed is marked `nw` so that
   nonneg.ail still counts its result as an unwrapped sum or product.
2. An `int` comparison whose operands' intervals give one answer becomes
   that constant; simplify then removes the dead branch (native.range.2).
3. `x / 2^k` and `x % 2^k` with `x` never negative are marked for the
   shift and mask of ADR 0816 (native.range.3).
4. `AIL_OPT_OFF=vrange` turns all three off; `vrcmp` keeps every comparison.

## Soundness

- An interval holds on every run reaching the definition. A checked
  operation that may overflow gets its interval clipped to `int`: past it,
  the operation didn't trap, so its exact result is in range. An unchecked
  one that might wrap is unknown.
- A fact `x < y` from a branch edge holds in blocks the edge's target
  dominates when that target has only that edge in: every path to them took
  the edge after `x` and `y` were last defined (their definitions dominate
  the branch). The other side's interval is its own, unrefined, which holds
  for any of its values. A fact that leaves an empty interval is ignored.
- The walk ends: parameters only grow (joined with their old interval), and
  each grows at most five times (three, then one jump to the end of `int` in
  each direction). If a function ever needs more rounds than that bound, the
  pass proves nothing for it.
- Where a check is not proved, it stays and traps at the same site with the
  same message (native.build.1). Tests `trap_vrange_join` (a byte on one path
  only), `trap_vrange_loop` (a doubling counter bounded from below only) and
  `trap_vrange_shift` (a count up to 127) cover kept checks;
  `vrange_checks` covers the removed ones.

## Rationale

ADR 0076 order: no source change, so tokens tie; AI understanding gains a
little (fewer surprises from `ail perf` listing checks that plainly can't
fire); speed is the criterion, and Option A buys the general case for the
least code and risk.

## Consequences

- Hash and mask arithmetic, counters with any constant step and
  count-down loops lose their overflow checks without a guard.
- `drop_bounded` in fold.ail and the counter rule of nonneg.ail become
  special cases of this pass; they stay until measurements show nothing is
  lost without them.
- Follow-up: relations between two values (difference bounds) would let the
  same pass drop `xs[i]` checks outside counted loops; F7's wider inlining
  gives it more to see.
- Measurements (F9 job reports, 2026-11-12; wall ms, best of 11, after/before):
  ail-linux x86-64: wordfreq 0.862; lzr 1.040 and vm 1.080 slower although
  lzr runs 0.5% fewer instructions and vm the same, so code layout, not the
  pass's output (lzr text 148,720 vs 156,448 bytes with the pass off).
  Mac arm64: lzr 0.989, wordfreq 1.032 (a 6 ms run), vm equal. The compiler
  is 4% smaller (10,320,065 vs 10,765,889 bytes). x86-64 loop placement is a
  follow-up; the release run re-measures vm and lzr against the pre-F9 tree.

## Rejected alternatives

- **Option B:** ranges.ail serves loop versioning; widening it would change
  the answers the guards rely on.
- **Option C:** too large and slow for 0.1.0; the bounds checks it would add
  are mostly covered by bce.ail and forward.ail.
