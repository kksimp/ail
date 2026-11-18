# 1141. Small branches become branch-free selects behind an overflow guard

- **Status:** Accepted
- **Date:** 2026-11-17
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for every speed change the reviews found that needs no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.select.1-2](../spec/23-native.md)
- **Amended by:** [ADR 1150](1150-select-chains-index-selects-unsigned-division.md) (chains; index selects that do not feed themselves)
- **Builds on:** [ADR 1063](1063-function-wide-value-ranges.md) (value ranges), [ADR 0623](0623-branch-and-codegen-peepholes.md) (`x % 2^k == 0` as a bit test), [ADR 0388](0388-arm64-code-generation.md), [ADR 0385](0385-x86-64-code-generation.md)

## Context

The round 2 codegen review (2026-11-17) found the whole collatz gap to C on
the Mac is one data-dependent branch: clang's loop picks `n / 2` or `3n + 1`
with `csinc` and has no branch on `n`; AIL's branches on the parity, which
mispredicts about one step in three. Neither backend could pick between two
values without a branch: no `select` in MIR, no `csel` or `cmov`.

A plain rule "convert when both arms can't trap" would almost never fire in
AIL, because integer arithmetic is checked (expr.arith.1): the odd arm
`3 * n + 1` has two overflow checks. Hand-written loop shapes on the Mac
(`perf-review/round2-codegen-mac.txt`, cycles per step): AIL today 4.72,
clang 3.35, a select with every check kept and a guard 3.19, the same with
no checks 3.36. The checks cost nothing measurable once they are off the
loop-carried chain, so "overflow always traps" stays as it is.

## Options considered

### Option A — select behind a range guard, original branch as the slow path (chosen)

A MIR pass (`compiler/mir/ifconv.ail`) finds a branch whose arms are a few
integer instructions that rejoin one block. For each checked `+`, `-`, `*`
or negation in an arm it computes the result as `c * x + d` of one input
`x`; that gives the range of `x` in which the operation can't overflow. The
branch block then compares each such input with its range; inside, both arms
run unchecked and a `select` per joined value picks the result; outside, the
original branch runs with its checked arms, unchanged.

- **Pros:** every check is kept: a trap happens exactly when it did, at the
  same site, with the same message. The guard is a predictable branch off
  the loop-carried chain. Fires on collatz, clamps, `min`/`max`, counting and
  best-so-far updates.
- **Cons:** up to four compares and some code on the slow path; an arm whose
  checked operation isn't affine in one input (`a - b` of two inputs) keeps
  its branch.

### Option B — trap-free arms only

- **Pros:** simplest.
- **Cons:** fires on almost nothing in AIL; not collatz.

### Option C — compute both arms with overflow flags and trap after the select

- **Pros:** no range reasoning; any checked add, sub or mul.
- **Cons:** a trap of the arm not taken must be ignored, so each flag has to
  be selected too; the trap then reports after the fact, and on arm64 a
  checked multiply needs `smulh` on the chain. More code per arm than a guard.

### Option D — change the overflow rule for loop counters

- **Pros:** fewer checks.
- **Cons:** changes a designer decision, and the Mac run shows the checks
  cost nothing in the select shape. Withdrawn by the review itself.

## Decision

1. MIR gets `select cond, a, b` (`imm` 1: `b + 1`; 2: `a + 1`, for `csinc`).
   It is pure. arm64 emits `csel` or `csinc`; x86-64 `cmov` after loading
   `a` into the result without touching the flags. A comparison, or a
   `x & m == 0` test, read only by the select right after it sets the flags
   the select reads, as for branches (ADR 0623).
2. The pass converts a branch when: each arm is the edge itself or a block
   that only the branch enters and that jumps on; both reach one block with
   at most three `int`, `bool` or `char` parameters; an arm has at most three
   instructions (constants and copies aside), both at most five; every
   instruction is integer arithmetic, a bit operation, a comparison, a shift
   by a constant or a division by a constant other than 0 and -1; and every
   checked operation is affine in one input with coefficients below 2^20 and
   2^40. At most two inputs get a guard. A branch that picks a value a list
   index is computed from, through integer arithmetic and block parameters,
   keeps its branch (`AIL_OPT_OFF=selindex` lifts this).
3. The guard of an input is `lo <= x <= hi`, rounded inward; a side that is
   the end of `int` is not compared. The slow path is the original branch
   with its condition's instructions moved next to it.
4. `x / 2^k` on the arm where `x % 2^j == 0` (`k <= j`) holds is an
   arithmetic shift, in the select and in the original arm (native.select.2).
5. An unchecked `n * 3`, `n * 5`, `n * 9` is one `add` with a shifted operand
   on arm64 and one `lea` on x86-64, so `3 * n + 1` costs two short
   instructions on the chain.
6. `AIL_OPT_OFF=select` turns the pass off. It runs last in `inline_module`,
   on user functions only.

## Soundness

- The fast path computes both arms on every input, so an arm may only hold
  instructions that can't trap or touch memory once its overflow checks are
  dropped; a division by a constant other than 0 and -1 and a shift by a
  constant in 0..63 can't trap.
- When an arm is selected, its instructions see the values they saw on the
  original path. An instruction whose check a value range removed
  (native.range.1) was proved exact on that path, so the affine form of a
  later checked operation is its exact value; the guard puts that value
  inside `int`. Values of the arm not selected are discarded, whatever they
  are.
- Outside a guard the original branch runs with its checks, so every trap
  is the one the program had.
- Tests: `select_branches` (picks, counters, best-so-far, a `bool`, exact
  halving of negative numbers, inputs on both sides of the guard) and
  `trap_select_guard` (an odd input past the guard traps as before).

## Rationale

ADR 0076 order: no source change, so AI understanding, readability and
tokens tie; speed decides. Option A is the only option that both fires on
checked arithmetic and keeps every trap exactly where it was.

## Consequences

- collatz's inner loop is `cmp; b.gt; cmp; b.lt; asr; add lsl; tst; csinc`
  plus the counter, against clang's seven instructions.
- A well predicted branch becomes a select too, which can lengthen a
  loop-carried chain by a cycle. The first Mac timing run (2026-11-17)
  showed it in `vm`: the jump-if-zero arm picks the next `pc`, which indexes
  the bytecode, so the next fetch waited for the condition (23% slower, same
  instruction count, 19% more cycles). A predicted branch lets that load run
  ahead, so a value that feeds an index keeps its branch. A profile-driven
  filter is the follow-up if another case shows.
- Follow-ups: one unsigned compare for a two-sided guard (needs an unsigned
  comparison in MIR); `float` selects (`fcsel`, `vblendv`).

## Rejected alternatives

- **Option B:** does not fire on checked arithmetic.
- **Option C:** more code per arm and traps that report late.
- **Option D:** changes a designer rule for no measured gain.

## Amendment (2026-11-18): the x86-64 `cmov` form, corrected

The first x86-64 lowering miscompiled a count. For `select c, n, n` with 1
added on the false side (`if not c: n += 1`, both sides one value) whose
result has `n`'s register, the rule "the result's register holds `b`, so
load `b` first and flip the condition" fired, moved the 1 to the first
side, added it in the shared register and then read the other side from
that register: the count went up every time (`tests/cases/heap.ail` printed
`bad 60`, `tools/gate/globals.ail` reported a slot shared). arm64 reads
both sides before it writes and was right. Commit 47186d86 replaced the
`cmov` by a short branch until the cause was known.

The `cmov` form is back. A side that adds 1 is computed into the scratch
register before the result is written and is the `cmov`'s source, so no
side is read after the result's register changes; a select of 0 and 0 plus
1 is `mov 0; setcc`. The operand swap remains only for a select with no 1
to add, and only when `b` alone is in the result's register. Test
`select_count_shared` (also run with `AIL_OPT_OFF=selsum`, which gives the
shape above). The original comparison, when only the removed branch read
it, is replaced by a constant, so no dead `setcc` or `cset` is left.

