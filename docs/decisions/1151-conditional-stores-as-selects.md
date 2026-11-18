# 1151. Conditional stores become selects behind an overflow test

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the open slowdowns against C, runtime speed only, no syntax change
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.select.3](../spec/23-native.md)
- **Builds on:** [ADR 1141](1141-branch-free-selects-with-guards.md), [ADR 1150](1150-select-chains-index-selects-unsigned-division.md), [ADR 0025](0025-memory-model-atomics.md) (safe code is data-race-free)

## Context

lzr trails its C twin (Mac, 12 MB input: 1.38 s against 1.10 s). Its range
coder takes one data-dependent branch per direct bit,
`if self.code >= self.rng: self.code -= self.rng`, which mispredicts half
the time; clang turns the same C into a conditional select. Hand-editing the
two `direct` methods of `lzr.ail` to pick with arithmetic measured 14% of
the whole program (`hotpaths-probe-darwin-41f563726.txt`: 1.513 s to
1.308 s on a loaded machine, C 1.216 s).

ADR 1141's pass cannot take these branches: the arm stores to a field, and
its checked `-` has two inputs, so no range of one input guards it.

## Options considered

### Option A — select the stored value, store always, test the overflow after the fact (chosen)

`compiler/mir/ifstore.ail`. Both arms are computed at the end of the
branch's block with their checks off. Each slot an arm stores gets
`select(cond, arm value, old value)` and one store that always runs. For
each checked `+` or `-`, the sign bits of operands and result say whether it
overflowed (`(x ^ s) & (y ^ s) < 0` for a sum, `(x ^ y) & (x ^ d) < 0` for a
difference); the tests are joined with `or`, and one predicted branch sends
an overflow to the original branch and its checked arms.

- **Pros:** fires on two-input arithmetic; every trap is the original
  one; no back end change (`select`, `bxor`, `band` exist).
- **Cons:** a store runs where the source had none, and three or four cheap
  instructions per checked operation.

### Option B — keep the object's fields in registers first (scalar promotion), then ADR 1141 as it is

- **Pros:** removes the loads and stores too.
- **Cons:** only for an object that does not escape; `Enc` is passed to
  calls. Still needs the overflow test for `code - rng`. A follow-up, not a
  replacement.

### Option C — branch-free arithmetic written in the source

- **Pros:** no compiler work.
- **Cons:** the benchmark source stays the plain one (ADR 0029 spirit: the
  compiler earns the speed), and user code would need the same trick.

## Decision

Option A, with these limits:

1. **When the added store is unseen.** A store to an object is added only
   when the branch's own block already stores to that object. The function
   therefore has write access to the object at that point whatever the
   condition says, and safe code is data-race-free (ADR 0025), so no other
   task can observe the slot between the load of its old value and the
   store of the same value.
2. **When a load may run early.** An arm's load runs before the condition
   is known only from an object the block itself reads or writes (the
   object exists), and only before any store of that arm.
3. Stored and joined values are `int`, `bool` or `char`. At most two slots,
   four instructions an arm, six in all. A value an index is computed from
   that feeds itself keeps its branch (ADR 1150).
4. Only `+` and `-` may be checked in an arm. The pass runs in the first
   round of ADR 1150's rounds only, so the slow path keeps its branch.
5. `AIL_OPT_OFF=selstore` turns it off.

ADR 0076 order: no source change, no diagnostic change; speed decides.

## Consequences

- `Dec.direct` and `Enc.direct` of lzr lose their data-dependent branch.
- A rarely taken, well predicted `if` that stores a field now loads,
  selects and stores every time. When the block's own store to the object
  is a read-modify-write of another field, the object's fields are already
  on such a chain; when it is not, the select adds one. Measurements decide
  whether a filter is needed (see `docs/status.md`).
- Follow-ups: element stores (`probs[idx] = ...` in `bit`); scalar
  promotion of stack objects with fields stored more than once (Option B).

## Rejected alternatives

- **Option B alone:** misses objects that escape to calls.
- **Option C:** moves the cost to every author.
- **Add the store wherever the object is merely read in the block:** a
  reader need not have write access; another task could hold the object.
