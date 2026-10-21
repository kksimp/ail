# 0845. Vectorization order: idioms first, a runtime fill and copy on both ISAs, `break` loops under a bit-identical rule

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 25 — Optimization strategy
- **Spec rules:** none (backend policy under conc.par.1)
- **Extends:** [ADR 0246](0246-simd-vectorisation.md), [ADR 0816](0816-native-codegen-entry-addressing-constants.md)

## Context

[The survey](../design/auto-vectorize-survey.md) measured the ten native-vs-C programs. `gcc -O2 -ffp-contract=off`, which the drag race uses, vectorizes no loop in any C twin; it turns the sieve's fill loop into `memset` and does a little two-lane SLP. The distance to C is scalar code and runtime (wordfreq 7.3x, fasta 2.4x, matmul 2.0x, binarytrees 2.0x in instructions), not SIMD. Vectorization would pay on three shapes: constant-fill and copy loops, mandelbrot (independent pixels with an early exit) and spectralnorm (independent rows with an in-order float reduction).

## Decision

1. **Order of work:** fill and copy idioms, then the scalar gap in wordfreq, then outer-loop float vectorization (mandelbrot with lane masks, then spectralnorm), then integer lanes. Integer NEON waits until a program needs it.
2. **Idioms are runtime functions on both ISAs.** A loop that stores one unchanging value, or copies one list's elements to another's, becomes a call to `rt.fill` or `rt.copy`. The AIL bodies run on ARM64 (and are the reference); x86-64 assembles them as `rep stos`/`rep movs` kernels (`x64/kernels.ail`). They are replaced by NEON kernels when those land. No ARM64 code generator change is needed.
3. **Loops with `break` may enter the independence verdict** (conc.par.2-4) for vectorization, on one condition: the vector loop's result is bit-identical to the scalar loop's. The proof is the interpreter-match test: every program with such a loop must print the same bytes natively and under `ail run`. If any program differs, `break` loops stay out of the verdict. Lane masks make the loop run until every lane has left; each lane's own operations are the scalar sequence.
4. **Unchanged from ADR 0246:** reductions are never reassociated, `a * b + c` is never fused, integer lanes check overflow per lane with a scalar redo.

## Rationale

Criterion 4 (speed) with no cost to the others, and the order follows the size of the gap. The idioms are what the C compiler already does, so they close a gap without a trick; outer-loop vectorization is the only item that makes native faster than the C twins by a wide margin. Lockstep (ADR 0008) is kept by giving ARM64 the same function now rather than waiting for NEON.

## Consequences

- `rt.fill` is a runtime root (one line in `lower.ail`'s `RT_ROOTS`); `rt.copy` is another.
- Instruction counts under callgrind undersell `rep` string instructions (one per iteration); the gate for these PRs is the interpreter match plus the instruction table, and wall time at a cached size.

## Rejected alternatives

- **Vectorizing before the scalar gaps close.** The scalar gaps are larger and cheaper to fix.
- **A pattern trick for the sieve's strided stores** (stride below 16). It would help one benchmark shape only.
- **Waiting for NEON before the idiom pass.** It would put the idioms behind an ARM64 kernel for no gain; the AIL body is the reference anyway.
- **Keeping `break` out of the verdict.** It would leave mandelbrot, the clearest showcase, scalar; the bit-identical condition keeps the semantics.
