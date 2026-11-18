# 1145. Native: `int` element loops in lanes with an overflow mask; vectors may be read across blocks

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for lanes on the
  remaining speed gaps (2026-11-17), runtime speed only, no syntax change, overflow traps kept
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.vec.6](../spec/23-native.md)
- **Extends:** [ADR 1142](1142-element-wise-vector-loops-and-list-alias-classes.md),
  [ADR 1144](1144-four-element-vector-passes.md); amends the "inside one block" rule of
  [ADR 1066](1066-pair-adjacent-float-operations.md)

## Context

After ADR 1144 the element-wise loops over `float` lists run four elements a pass. The same loops over `int`
lists (`c[i] = a[i] + b[i]`, `xs[i] = xs[i] + 1`) stayed scalar: an `int` `+` or `-` is checked (expr.arith.1),
and a C compiler vectorizes its twin only because overflow is undefined there. The check is the whole question:
a lane operation wraps, and a trap must still come at the iteration, and with the message, of the scalar loop.

ADR 1144 also left the splat of a value from before the loop (`k` in `a[i] * k`) inside the loop, made again
each pass, because a vector could not leave the block that made it.

## Decision

1. **`int` lanes.** A loop of native.vec.4 may have `int` elements, with `add`, `sub`, `band`, `bor` and `bxor`
   in its body. The other operand may be a constant or a value from before the loop, never a counter. A body is
   all `float` or all `int`. The vector types hold either kind (64 bits a lane); the operations are `viadd
   visub vand vor vxor`, and `vsplat` of an `int`.
2. **The overflow mask.** For each checked `add` (`r = a + b`) the pass computes `(a ^ r) & (b ^ r)`, for each
   checked `sub` (`r = a - b`) `(a ^ b) & (a ^ r)`: negative in the lanes that overflowed. The masks of a pass
   are ORed, and `vsign` gives an `int` that is not 0 when any lane is negative (`vmovmskpd` on x86-64; the two
   lanes ORed and shifted on ARM64).
3. **Test, then store.** The body of a vector loop becomes two blocks: the first computes the lanes and the
   mask and tests it; the second holds the stores. A pass with an overflow writes nothing and leaves the vector
   loops with its counters untouched, so the original scalar loop runs from that iteration and traps there, as
   it always did. A checked operation after a store in the body keeps the loop scalar.
4. **Vectors across blocks.** A `Vec2` or `Vec4` may be read in a block that its definition dominates. It is
   still never a block parameter and never live across a call. The allocator's liveness already covered this;
   `vec4.ail` now tracks the halves over the whole function.
5. **Splats once.** Constants of the body and splats of values from before the loop are made in the block in
   front of the vector loops, for `float` and `int` loops alike.
6. **Knob.** `AIL_OPT_OFF=vmap`, as before.

## Rejected alternatives

- **Store first and test after.** Shorter (one block), but a pass that overflows would have written wrapped
  values; a loop that writes the list it reads would then run its scalar iterations on changed input.
- **Wrapping lanes with no check where a range proof exists only.** Covers few loops; most element values have
  no known range.
- **Saturating or widening tricks for the check** (`sqadd` on ARM64 and compare). No x86-64 form below
  AVX-512; the three bitwise operations are the same on both.
- **`mul` in lanes.** Neither AVX2 nor NEON multiplies 64-bit lanes, and its overflow test needs the high
  half. Left scalar.
- **Counters as lane operands (`c[i] = a[i] + i`).** Needs a vector of the counter's values; left for later.
- **Keeping vectors block-local and recomputing in the store block.** Twice the loads and arithmetic.

## Result

ail-linux (x86-64) at bd4202b, best of 5, ms at N = 1000 / 4000. `vint` runs `c[i] = a[i] + b[i]` and
`xs[i] = xs[i] + k` over `int` lists 200,000 times; "off" is `AIL_OPT_OFF=vmap,fwdalias,fwdloop,fwdhoist`.
Outputs identical on and off; `vmap_int.out` was computed independently.

| Program | on | off |
|---|---|---|
| vint | 124.3 / 542.5 | 285.8 / 1074.9 |
| axpy | 87.2 / 515.6 | 313.3 / 1191.6 |
| mapcomp | 157.4 / 555.6 | 193.2 / 684.5 |

The C twin of `vint` at N = 1000, which checks nothing: gcc -O2 164.5, -O3 111.7, -O3 with
`-march=x86-64-v3` 52.1; with `-ftrapv` (checked, as AIL is) 794.8. The checked lanes cost about three vector
operations and one test a pass over the unchecked ones.
