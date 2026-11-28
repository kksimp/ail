# 1160. Loops over structs kept in place: checks out of the loop, constants held, one-instruction addresses

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the speed gaps still open
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.bce.18](../spec/23-native.md); none for the two code-generation items (instruction selection; no behaviour changes)
- **Builds on:** [ADR 1149](1149-structs-kept-in-place-in-lists.md) (structs in place), [ADR 0577](0577-bounds-checks-in-counted-loops.md) (loop versioning), [ADR 1154](1154-arm64-multiply-add.md) (its open item, constants a loop uses kept in registers, is done here for `float`s)

## Context

`aosmut` updates two fields of each struct of a `List[Body]` in a counted
loop. It was reported 2.5 times behind its C twin. That figure compared
with `gcc -O3`, which swaps the benchmark's 100-pass loop with its element
loop and so streams the list once, not 100 times; a twin with the pass kept
out of line (`aosmut_fair.c`) is the fair one. Against it AIL was 1.12 times
behind on ail-linux and 1.7 times behind on the Mac, where the loop was 18
instructions for clang's 9:

- `ps[i].x` lowers to `bounds i, len ps` and the address `data + i * 32`
  (ADR 1149). Loop versioning (ADR 0577) removes the checks of `get` and
  `set` only, so this check stayed in the loop.
- The code generators rebuild a constant at each use. `0.001` and `0.999`
  were a literal load and a move each on arm64, a load each on x86-64, on
  every pass.
- The address was a shift and an `add` on arm64, and `imul` by 32 and an
  `add` on x86-64.

## Options considered

### Option A — three small changes where each cost arises (chosen)

1. **Checks (native.bce.18).** In `version` of `compiler/mir/bce.ail`, a
   `bounds idx, len x` is the access of `x` at `idx`, classified as a `get`
   is (the counter, the counter plus a constant, a fixed value, ...). The
   guard tests the length once and the copy of the loop that runs under the
   guard has no `bounds`. The original loop is unchanged, so a trap is the
   one it was.
2. **Constants.** A last pass before register allocation,
   `compiler/mir/holdconst.ail`, gives each non-zero `fconst` read in a loop
   with no call a `copy` in the block before the loop; the loop reads the
   copy, which the allocator holds in a register like any value. It is a MIR
   pass, so both targets get it (ADR 0008).
3. **Addresses.** arm64: an unchecked `x * 2^k` read only by the unchecked
   `add` after it is `add d, a, x, lsl #k`, through the fusion ADR 1154
   added for `madd`. x86-64: an unchecked `x * 2^k` is a shift, not `imul`
   (`lea` has no scale of 32, so the `add` stays).

- Pro: each is a few lines in a place that already does its kind of work.
- Con: the constants are held only for loops without calls, and only
  `float`s; an `int` constant is an immediate in most uses.

### Option B — a strength-reduced pointer that steps by the struct size

- Con: it needs an induction-variable pass the compiler doesn't have, and
  after (3) the address is one instruction on arm64 anyway.

### Option C — vector lanes over whole structs

- Con: the fair C twin is not vectorized either, and the loop is bound by
  memory at the sizes that matter; blends and permutes on both targets for
  no measured gap.

## Decision

Option A. `AIL_OPT_OFF=bceflat`, `holdconst`, `addshift` and `mul3` turn the
parts off. `holdconst` keeps out 0.0 (one instruction that reads nothing),
constants passed on an edge, and more than six constants a loop; inner
loops go first, so a constant is rebuilt once per run of the innermost loop
that reads it.

## Consequences

`aosmut`, a million structs, 100 passes, the change against the main it
merged, best of 5, ms (2026-11-18, `vwide4-job-mac-fbcac78.txt`,
`vwide4-job-linux-fbcac78.txt`):

| | before | after | C, pass out of line | C `-O2` |
|---|---:|---:|---:|---:|
| Mac (arm64) | 113.3 | 76.1 | 66.6 | 66.6 |
| Mac, 20,000 structs (fits the cache) | 4.0 | 3.2 | | |
| ail-linux (x86-64) | 325.6 | 317.2 | 317.8 | 289.7 |

On the Mac the loop went from 18 instructions to 12 and from 1.7 to 1.14
times the C twin. On ail-linux the loop is bound by memory and gains 3%.
`aos`, `aosfor` (read-only loops with no constants), `axpy` and `mapcomp` are
unchanged on both. The removal of the check alone, measured first on the Mac
(`vwide4-job-mac-088636c.txt`), changed no time: the gain there is the
constants and the address. 62 programs give the same output with the parts
on and off, and the compiler reaches its fixed point, on both machines.

## Rejected alternatives

Options B and C above. Holding `int` constants too: on arm64 only those
that need more than one instruction would gain, and the one known case
(`mapseed`'s multiplier) is level with C since ADR 1154. Holding constants
across calls: it would take callee-saved registers from values that need
them more.

## Amendment 2026-11-28: wide integer constants (kksimp/ail#1598 item 5)

`compiler/mir/holdconst.ail` holds a `const` outside the loops that read it when it does not fit 32 bits
(x86-64 `movabs`, up to four arm64 `movz`/`movk`) and the loop reads it at least twice (`MIN_WIDE_USES`), under the same
`MAX_HELD` limit and call-free rule as `float` constants. Reads by `div`, `rem`, `mul` and shifts keep the constant,
because their generators pick a cheaper form from it. Rejected: hoisting every constant (register pressure, small
constants are one instruction), a hook in `alloc.ail` (the copy is an ordinary value, so the allocator already keeps
it in a register or a slot). Test: `tests/native/run/holdconst_wide.ail`. Not built or measured in the cloud (Mac gate).
