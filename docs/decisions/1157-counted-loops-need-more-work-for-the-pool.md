# 1157. A loop whose work is all counted needs 8388608 operations for the pool; division and remainder cost more

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer ("keep iterating on your own until we are close to on par with C", 2026-11-17)
- **Topic:** 11 — Concurrency
- **Spec rules:** amends [conc.par.14](../spec/11-concurrency.md)
- **Builds on:** [ADR 0911](0911-default-automatic-parallel-loops.md), [ADR 0946](0946-adaptive-thread-pool.md), [ADR 0987](0987-auto-parallel-inner-loops.md)

## Context

A loop the compiler chose went to the pool past 65536 estimated operations,
which ADR 0911 took as about 50 microseconds. For a body of plain arithmetic
and list reads that is wrong by an order of magnitude: the backend vectorises
such loops (ADRs 1142, 1144), so an operation costs well under a tenth of a
nanosecond, while starting a region costs 25 to 35 microseconds.

Measured on main b6896866, 2026-11-18, as pool time over in-order time for a
loop of N iterations run again and again (ail-linux, 4 cores / the Mac, 8):

| Loop body | Counted ops | N = 12000 | Pool first wins at N | N = 1024000 |
|---|---|---|---|---|
| `c[i] = a[i] * k + b[i]` (float) | 7 | 5.89 / 6.45 | 512000 / never | 0.91 / 1.23 |
| int additions | 6 | 3.52 / 5.10 | 256000 / 512000 | 0.82 / 0.95 |
| float, two divisions | 17 | 2.34 / 4.51 | 128000 / 256000 | 0.81 / 0.77 |
| int, a branch, `/` and `%` | 8 | 1.21 / 2.14 | 24000 / 64000 | 0.44 / 0.35 |

Cheap operations repay the pool at 2 to 4 million counted operations or, for
a loop bound by memory, not at all; a body that divides repays it at about
0.2 to 0.5 million, because the count priced a division as one operation.

The loops that gain most from the pool (ADR 1143's parfold, fieldpar, collatz
and raytrace) all have a loop or a call of a function with one in the body.
Their count is only a floor, and they must keep the low bar.

## Decision

1. **Counted loops.** A loop the compiler chose is *counted* when the count
   has all of its work: the body, and every function it reaches, has no loop,
   no recursion, no call of a method other than a constant-time read-only one
   (`join` and `chars` are not), no call of a function with no body in the
   program, and builds no interpolated string and no list, map or set
   literal. A counted loop goes to the pool only at 8388608 operations in all,
   however heavy one iteration is. Every other loop keeps 65536 operations in
   all and 4096 an iteration.
2. **Division and remainder.** In the count a `/` is 16 operations and a `%`
   is 64; by an integer literal, where the backend multiplies, they are 4 and
   6. `x /= e` and `x %= e` count the same.

The scan sets `ParScan.opaque`; the lowering keeps which functions have a
cost that is only a floor (`TaskState.fvague`) and passes `rt.par_worth` 2
for a counted loop. The loop inside a loop run in order uses the same number
in its inline test.

## Rejected alternatives

- **Raise the one threshold to about 2 million for every loop.** A loop whose
  body calls a method or a function with a loop has work the count can't see;
  it would need 30 times more iterations and the pooled loops of ADR 1143
  would run in order at ordinary sizes.
- **Price operations by type.** The scan works on syntax, shared with
  `ail check`, and has no types. A `%` is nearly always of integers, and the
  float-division case measured above is served by 16.
- **Make a region cheaper to start (workers that spin before they sleep).**
  It would lower every break-even, but it burns a core between regions and
  changes the pool for every program. Left for a measured change of its own.
- **Measure the loop at run time and adapt.** Exact, but it adds state and a
  clock read to each region and makes timings depend on history.
- **Fewer workers for small counted regions.** At 8388608 operations the rule
  of one worker per 65536 already gives the whole pool; nothing to tune.

## Consequences

Short arithmetic loops run in order, as fast as before ADR 0911 turned the
pool on. A counted loop between the old and the new threshold that would have
gained from the pool now runs in order: with the weights, the integer
division loop above reaches the pool near 100000 iterations rather than
24000 to 64000, a missed gain of up to 1.6x in that band, taken to avoid
losses of up to 10x. A loop bound by memory, like the float one above, can
still lose a little on the pool at any size. The measurements of this change
on Linux and macOS are in [status.md](../status.md).
