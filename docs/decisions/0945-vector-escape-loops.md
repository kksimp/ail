# 0945. Native: vector kernel for escape-time loops (ADR 0845 stage 2)

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 23 — Implementation (native optimizer)
- **Spec rules:** [native.vec.2](../spec/23-native.md)
- **Extends:** [ADR 0845](0845-vectorization-order-and-idioms.md) (outer-loop float vectorization with lane masks), [ADR 0942](0942-user-loop-vector-kernels.md) (idiom kernels on both ISAs)

## Context

ADR 0845 ranked mandelbrot the clearest showcase for vectorization: 96% of its
time is the escape loop, the points are independent, and every operation is a
float multiply, add, subtract or compare, so a lane mask gives results
bit-identical to the scalar loop. The scalar code is a dependent chain per
point (about 4 cycles of multiply and add latency a step), so one point at a
time leaves most of the unit idle.

## Decision

1. **An idiom pass, like ADR 0942's, not a general vectorizer.**
   `compiler/mir/vecloop.ail` matches, in the MIR after inlining, a counted
   `x` loop from 0 whose body makes `cr` from float arithmetic of `x` and
   outside values and then runs the exact escape loop of native.vec.2
   (constant `MAX`, `ci` from before the `x` loop, start at `0.0`). The
   operation order is the match's condition, so the lanes compute what the loop
   computes. Anything else runs as written.
2. **Versioned by a guard, one buffer pair per execution of the `x` loop.** If
   `n <= 2^24`, a new loop computes every `cr` into a buffer (a copy of the
   slice of the body that makes it; float arithmetic cannot trap), one call of
   `rt.vec_escape(cr, out, n, max, keep; ci, thr)` fills the counts, and the
   `x` loop's body takes its `k` from the buffer in place of the `while` loop;
   the escape loop's blocks stay for the other path, which the guard selects
   when `n` is larger. The exit block gains a parameter for `k`. Both buffers
   are `rt.list_new` lists with room for `n` rounded up to 4.
3. **Kernels on both ISAs** (ADR 0008). x86-64: four lanes in `ymm`; live
   lanes `m`; each step `s = zr*zr + zi*zi`, `m &= ~(s > T)`, stop when no lane
   lives, `t = (zr*zr - zi*zi) + cr`, `zi' = (2*zr)*zi + ci`, a blend that keeps
   dead lanes' `zr`, `zi`, and a count held as a double (`k += m & 1.0`),
   converted to `int` at the end (`MAX` is at most 2^20, exact). ARM64: the same
   on `.2d` with `fcmgt`, `bic`, `bsl`, `addp` for the any-lane test and
   `fcvtzs`. The compare is false for a NaN, as `fgt` is, so a NaN lane never
   leaves, as in the scalar loop. No FMA, operand order as in the loop.
4. **Opt-out:** `AIL_OPT_OFF=vec` (also turns off the ADR 0942 kernels) or
   `vecloop` runs the scalar loop. The AIL body of `vec_escape`
   (`compiler/rt/native/vecint.ail`) is the reference; the test
   `tests/native/run/vec_escape.ail` compares against the output of the scalar
   loop.

## Rationale

Criterion 4 (speed) with no token, semantic or readability cost, in the
shape ADR 0942 proved: a small matcher that builds on the existing MIR
infrastructure and a kernel per ISA, instead of vector types in MIR and the
register allocator. The cost is generality: a differently written escape loop
(another exponent, `abs` of a complex number as a struct) runs scalar until the
matcher knows it. The lane-masked kernel is the part a general vectorizer would
reuse.

## Consequences

- Spectralnorm (vectorizing over independent rows) is the next stage-2 target
  and needs a different shape (a reduction per lane), left for its own change.
- The `x` loop's preheader now has the guard and join blocks, so the loop's
  own versioned bounds-check removal sees the join block as its preheader.

## Rejected alternatives

- **Vector types in MIR and the allocators:** the right long-term shape, a
  much larger change touching both register allocators; revisit when a second
  loop shape needs it.
- **Replacing only the call to `escape`:** one point a call cannot be vectorized.
- **Scalar tails in the kernel:** buffers with room for a whole last group are
  simpler and the lane mask makes the extra lanes harmless.
