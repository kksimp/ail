# 1162. `if c: n += w` is a checked sum of a select

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns against C, runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.select.4](../spec/23-native.md) (new)
- **Builds on:** [ADR 1141](1141-branch-free-selects-with-guards.md) (selects with guards), [ADR 1150](1150-select-chains-index-selects-unsigned-division.md)

## Context

A count (`if c == 'e': total += 1`) is the commonest small branch. ADR 1141
turns it into `select c, total + 1, total`, and because `total + 1` is
checked, puts a guard before it (`total <= max - 1`) with the original
branch as the slow path. On x86-64 that was about twelve instructions an
iteration (`textgap-linux-15371b87.txt`: a dead `sete`/`movzbl`, a 64-bit
constant and compare for the guard, and the select) against four for the
plain branch; `charat` ran at 4.7 times gcc and `charwalk` at 4.1.

## Options considered

### Option A — add `select(c, w, 0)` with the sum still checked (chosen)

`compiler/mir/ifsum.ail`, tried before the general conversion. The branch
goes; the block ends with the comparison, `m = select c, w, 0` and the
original `add`/`sub` of `n` and `m`, at its source place. For `w` of 1 the
select is 0 or 1 from the condition (`cset`; `mov 0; setcc`).

- **Pros:** adding 0 cannot overflow, so the trap fires exactly when the
  branch would have been taken and overflowed, with the same message; no
  guard, no slow path, no second copy of the code; any `w` computed before
  the branch, not only constants; both targets.
- **Cons:** the overflow jump stays in the loop (never taken, predicted).

### Option B — keep the guarded select and make it shorter

- **Cons:** still a guard and a second copy of the branch per count.

### Option C — leave counts as branches on x86-64

- **Pros:** four instructions when the branch predicts.
- **Cons:** a count over data mispredicts, which is what ADR 1141 removed;
  the two targets would differ.

## Decision

Option A (native.select.4). x86-64: `cmp; mov 0; setcc; add; jo`. arm64:
`cmp; cset; adds; b.vs`. A sum that cannot overflow (no check) keeps
`csinc`/`cmov` from ADR 1141, which is shorter. `AIL_OPT_OFF=selsum` turns
it off.

## Rationale

ADR 0076 order: no source change and the same diagnostics, so AI
understanding, readability and tokens tie; speed decides, and A is shorter
than B on both targets without C's mispredictions.

## Consequences

- Tests `select_sum`, `trap_select_sum`.
- Mac (2026-11-18, against main 7661fb5a, `hotpaths-sel-mac-959f3c2db.txt`):
  charwalk 0.451 s to 0.304 s, charat 0.766 s to 0.674 s, collatz 0.441 s
  to 0.381 s (collatz from the dead comparison of ADR 1141's amendment).
- ail-linux x86-64 (`hotpaths-sel-linux-959f3c2db.txt`): charwalk 1.651 s to
  0.691 s (0.863 s with `AIL_OPT_OFF=selsum`), charat 1.574 s to 1.429 s.
  With the `cmov` form of ADR 1141's amendment: fasta 1.423 s to 0.605 s (C
  twin 1.005 s), collatz 1.786 s to 1.098 s, lzr 3.043 s to 2.846 s.
- charwalk's count repeats every ten characters, so its branch predicts
  perfectly and `AIL_OPT_OFF=select` is faster there (0.587 s); a count over
  data that does not repeat mispredicts, which is the case selects are for.
- Follow-up: `if c: n *= w` as `n * select(c, w, 1)`.

## Rejected alternatives

- **Option B:** more code for the same result.
- **Option C:** mispredicts on data and splits the targets.

## Amendment (2026-11-21): several sums and picks in one arm

The sieve's count loop (`if is_prime[k]: count += 1; total += k; largest = k`)
has an arm of two checked sums and an assignment, so the pass did not fire and
the loop kept its branch (8 million misses at size 100000000), where clang
compiles it with `cset`/`csel` and no branch. This is the whole Mac gap in
that row (1.16 against clang; `b1-gaps/sieve.md`).

`ifsum.ail` now accepts an arm of up to three checked sums plus plain
assignments of values computed before the branch. Each sum becomes
`n + select(c, w, 0)` at its source place (still checked, so it traps exactly
when the branch would have), each assignment `select(c, value, old)`, and
every select has its own copy of the comparison right before it, which is
what the flag-fusing code generators need. A join value that feeds a list
index keeps the rule of native.select.1. An arm with no sum is left to the
general conversion (`ifconv.ail`).

- **Rejected:** a select of the whole arm's results behind a guard (option B
  again, with three guards); a longer arm limit (a predictable branch over
  many operations is faster than the selects that replace it).
- Tests: `select_sums`, `trap_select_sums`.
