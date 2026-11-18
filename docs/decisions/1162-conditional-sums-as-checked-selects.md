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
- Not yet measured; `charat` and `charwalk` on ail-linux are the check.
- Follow-up: `if c: n *= w` as `n * select(c, w, 1)`.

## Rejected alternatives

- **Option B:** more code for the same result.
- **Option C:** mispredicts on data and splits the targets.
