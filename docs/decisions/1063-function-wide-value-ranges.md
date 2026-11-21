# 1063. Function-wide value ranges decide checks and comparisons

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer put every speed slice in 0.1.0 on 2026-11-12
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.range.1-3](../spec/23-native.md)
- **Amended:** 2026-11-21 (index checks as facts, late pass; see the amendment)
- **Extended by:** [ADR 1150](1150-select-chains-index-selects-unsigned-division.md) (native.range.3: any constant divisor)
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

## Amendment 2026-11-21: index checks as facts

The 0.2.0 gap study (vm on x86-64, 24 instructions a VM step against 11 to 13
for gcc and clang) found that about half of a step is checks and moves that
an earlier check already settles: `sp - 1` and `sp - 2` keep their overflow
checks right after `st[sp]` passed, the same `st[sp - 1]` is tested twice
against two copies of the constant 64 (constlen.ail gives each access its own
`const`), and hoist.ail's `copy` of the element pointer costs a `mov` before
each access.

1. An index check that passed (a list `get` or `set` with a trap site, or a
   `bounds` check) is a fact `0 <= i < len` about its index. It holds after
   the check in its block and in every block that block dominates; the walk
   tracks the instruction it is at, so an instruction before the check never
   uses it. Without a length value the upper end is 2^48 - 1 (as `len()`).
   At most 8 such facts are kept per value, so the walk stays linear.
2. A `bounds` check goes when its index's interval is within `0..len`'s lower
   end, or when a check of the same index against the same length value
   already holds there (native.range.1).
3. `late_checks` (compiler/mir/latecheck.ail) runs after the headers are
   hoisted: hoist.ail's `copy` of a `len` or `data` value is replaced by the
   value itself, then the walk runs once more with only point 2 and the
   overflow checks of point 1 of the decision (`late_ranges`); no comparison
   is folded there.
4. `AIL_OPT_OFF=vrcheck` turns off the index facts; `latecopy` and
   `lateranges` turn off the two halves of the late pass.

Soundness: an index check traps unless `0 <= i < len` (type.coll.4), so past
it the fact holds; `i` is an SSA value, so it holds wherever the check
dominates. A removed `bounds` check could never trap, because one with the
same operands, or with a length that was no larger, passed before it; every
trap that can still happen is at its own site with its own message. Tests:
`vrange_index` (removed checks), `trap_vrange_index` (an index below 0 still
traps at its access).

Rejected: merging `code[i]` and `code[i + 1]` into one test of `i + 1`
(B1's item 1b): it needs `i >= 0` known, or a cold path that repeats the
first test so the message is the same; left for a later change. Dropping the
check of `steps += 1` is a rule question, not an optimization.

## Rejected alternatives

- **Option B:** ranges.ail serves loop versioning; widening it would change
  the answers the guards rely on.
- **Option C:** too large and slow for 0.1.0; the bounds checks it would add
  are mostly covered by bce.ail and forward.ail.
