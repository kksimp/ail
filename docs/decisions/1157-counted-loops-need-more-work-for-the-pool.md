# 1157. A loop whose work is all counted needs 1048576 operations for the pool, an element-wise float store 16777216; division and remainder cost more

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

The first three bodies are one element-wise store. In order they run in
vector lanes (native.vec.4 to native.vec.6); the chunk function of the pool
runs them one element at a time, so the pool starts several times behind,
and at sizes bound by memory it never catches up. The fourth is scalar either
way (its divisors are literals: `x % 3`, `x / 7`, `x * 5 % 11`). One number
does not fit both kinds (measurements and analysis shared by the ADR 1144
lane, `vwide/parsweep.ail` in the project library).

The loops that gain most from the pool (ADR 1143's parfold, fieldpar, collatz
and raytrace) all have a loop or a call of a function with one in the body.
Their count is only a floor, and they must keep the low bar.

## Decision

1. **Counted loops.** A loop the compiler chose is *counted* when the count
   has all of its work: the body, and every function it reaches, has no loop,
   no recursion, no call of a method other than a constant-time read-only one
   (`join` and `chars` are not), no call of a function with no body in the
   program, and builds no interpolated string and no list, map or set
   literal. A counted loop goes to the pool only at 1048576 operations in all,
   however heavy one iteration is. Every other loop keeps 65536 operations in
   all and 4096 an iteration.
2. **Element-wise float stores.** A counted loop over a range whose body is
   one statement `c[i] = e`, where `c` is a list of floats and `e` is made of
   `+`, `-`, `*`, `/`, elements `x[i]`, names and number literals, needs
   16777216 operations. The shape is found by syntax (`ast.lanes_target`) and
   the element type by the lowering.
3. **Division and remainder.** In the count a `/` or a `%` is 16 operations;
   by an integer literal, where the backend multiplies, they are 4 and 6.
   `x /= e` and `x %= e` count the same.

With these the four bodies above reach the pool at about 2400000, 150000,
380000 and 50000 iterations.

A first version (8388608 for every element-wise store, `%` 32) was measured
on the Mac (8 cores, tip 8a76f264 against main 7661fb5a): short loops ran in
order as intended (float store of 12000 elements 6.5x faster than main), but
the int store `c[i] = a[i] + b[i] + s` stayed in order to 1200000 elements
though the pool wins from 192000 (it is not in vector lanes on main); the
float store on the pool at 2048000 elements was 11% slower than in order;
and `(a[i] + s) / (b[i] + 3) + a[i] % (b[i] + 7)` went to the pool at 24000,
28% slower than in order there. Hence floats only, 16777216, and `%` 16.

A loop that builds a list by `push` is never counted (a `push` is a method
the count can't price), so the builder rule of ADR 1155 is unchanged; its 16
and 256 are in the same count, where a `/` or `%` now weighs more.

The scan sets `ParScan.opaque`; the lowering keeps which functions have a
cost that is only a floor (`TaskState.fvague`) and passes `rt.par_worth` 2
for a counted loop and 3 for an element-wise store. The loop inside a loop
run in order uses the same numbers in its inline test.

### Amended 2026-11-18: division by a value on x86-64

Measured on ail-linux after the merge (4 cores, main f2fb2e12 against
7661fb5a, `parcost-linux.txt`): the cliff is gone for every body (float store
of 12000 elements 482 ms to 68 ms for the same elements in all), but
`(a[i] + s) / (b[i] + 3) + a[i] % (b[i] + 7)` costs 20 ns an iteration in
order there against 1.2 ns on the Mac, so at 45 counted operations it stayed
in order to 23000 iterations though the 4-core pool is 1.8x faster from
8000 (534 ms against 986 ms). The hardware divides differ by more than ten
times; one price can't serve both.

The scan also sums `ParScan.slow`: 16 for each `/` and 80 for each `%` in the
loop's own body whose right side is not an integer literal. `rt.par_worth`
takes it as a fifth argument and adds it to the work of a counted loop when
the target is x86-64, so there a `/` by a value counts 32 and a `%` 96, and
the body above reaches the pool at about 7400 iterations. The test inside a
loop run in order, which is compiled for every target alike, uses the larger
count; the runtime decides after it. Divisions inside called functions are
not added. A float `/` is overpriced on x86-64 by this (the scan has no
types): a float store with two divisions reaches the pool at 220000
elements there instead of 381000, where the pool measured level or ahead.

## Rejected alternatives

- **Raise the one threshold for every loop** (2 million; 524288 was tried in
  the ADR 1144 lane). A loop whose body calls a method or a function with a
  loop has work the count can't see; it would need many times more iterations
  and the pooled loops of ADR 1143 would run in order at ordinary sizes.
- **One number for every counted loop.** At 8388608 the scalar body above
  stays in order to about 380000 iterations and gives up a gain of 1.5x to
  2.1x; at 1048576 the float store goes to the pool at 150000 elements, where
  it is 1.4x to 1.9x slower.
- **The higher number for int stores too.** They are not in vector lanes in
  order on main; when native.vec.6 lands the element test should take them
  and the number be measured again.
- **Price operations by type.** The scan works on syntax, shared with
  `ail check`, and has no types. A `%` is nearly always of integers, and the
  float-division case measured above is served by 16.
- **Run the chunk function in vector lanes too.** It would lower the
  element-wise number, though not for sizes bound by memory. A change to the
  vectoriser, left to its lane; the number is to be measured again then.
- **Make a region cheaper to start (workers that spin before they sleep).**
  It would lower every break-even, but it burns a core between regions and
  changes the pool for every program. Left for a measured change of its own.
- **Measure the loop at run time and adapt.** Exact, but it adds state and a
  clock read to each region and makes timings depend on history.

## Consequences

Short arithmetic loops run in order, as fast as before ADR 0911 turned the
pool on. An element-wise float store between about 200000 and 2400000 elements
that would have gained 5% to 20% from the pool on some machines now runs in
order, taken to avoid losses of up to 10x below that. A store bound by
memory can still lose a little on the pool at any size. The measurements of
this change on Linux and macOS are in [status.md](../status.md).
