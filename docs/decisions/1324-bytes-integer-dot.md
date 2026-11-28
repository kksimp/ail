# 1324. `a.dot_i8(b)` and `a.dot_u8i8(b)` on `bytes`: exact integer dot products

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer made ail-llm speed top priority (kksimp/ail#1598) and approved no new syntax or type for it
- **Topic:** 04 — Types
- **Spec rules:** [type.bytes.19](../spec/04-types.md)
- **Relates to:** [ADR 1316](1316-list-dot-and-mul-add.md) (`dot` on lists of floats), [ADR 0005](0005-x86-64-baseline-v3.md), [ADR 0006](0006-arm64-baseline-v8-2.md), [ADR 0008](0008-isa-priority-policy.md)

## Context

After the compiler speedups of #1598, a pure-AIL llama-style runtime (ail-llm) is compute-bound in its weight
kernels. Q8_0 and the K-quants are blocks of 8-bit weights times 8-bit activations, summed as integers and scaled
once per block; the weights come from `fs.map` as `bytes`. AIL's best scalar form (two products in one `imul`) runs
near 2.9 GMAC/s a core, an order of magnitude under what a vector unit does. Integer addition is exact, so unlike
ADR 1316 there is no summation order to define: every path gives the same bits by construction.

## Decision

- **`a.dot_i8(b)`** on `bytes` is the `int` sum of `a.i8(i) * b.i8(i)`; **`a.dot_u8i8(b)`** is the sum of
  `a.u8(i) * b.i8(i)` (K-quant weights are unsigned nibbles against signed activations). Different lengths trap
  (`type.bytes.19`).
- **Slices read in place**, as `dot` does: `w[a..b].dot_i8(x[c..d])` with each sliced operand a variable or a field
  checks the bounds (`type.bytes.4`) and copies nothing. A block of 32 is one call.
- **Kernels** (`rt.vec_dot_i8`, `rt.vec_dot_u8i8`): x86-64 AVX2 (`vpmovsxbw`/`vpmovzxbw`, `vpmaddwd`, `vpaddd`;
  x86-64-v3 is the baseline, ADR 0005) and ARM64 NEON baseline (`smull`/`sadalp`; `ushll`/`sshll`/`smlal`). The
  runtime (`rt/native/bdot.ail`) runs them over blocks of 32 bytes in runs of at most 2^18 bytes so the 32-bit lane
  sums cannot overflow, adds the tail in scalar code and the runs as `int`. The AIL bodies of the kernels are
  their definition.
- No new type and no syntax: two methods on `bytes`. The names say the element kinds, as `u8(at)` and `i8(at)` do.

## Measurements

x86-64 Linux (cloud Xeon, one core), `tools/perfbench/micro/dot_i8.ail`, ns per multiply-add:

| n | loop over `a.i8(i) * b.i8(i)` | `dot_i8` | `dot_i8` of slices | `dot_u8i8` | `dot_i8` against the loop |
|---|---|---|---|---|---|
| 32 | 0.36 | 0.065 | 0.047 | 0.045 | 5.6x |
| 64 | 0.36 | 0.028 | 0.037 | 0.037 | 12.7x |
| 1,024 | 0.29 | 0.012 | 0.014 | 0.013 | 24x |
| 4,096 | 0.26 | 0.012 | 0.009 | 0.009 | 23x |
| 1,048,576 (memory bound) | 0.41 | 0.065 | 0.065 | 0.065 | 6.3x |

A block of 32 through the slice form costs 1.5 ns, about 21 GMAC/s. The ARM64 kernels give the same bits as the
x86-64 ones under qemu (`tests/native/run/bytes_dot.ail`, 20 lengths up to 600,001 bytes and the extremes against
the loop written out); Mac timings come with the gate run.

## Rejected alternatives

- **`List[i8]` operands.** A list holds each element in 8 bytes, so a kernel would read 8 times the memory of the
  `bytes` that mmapped weights already are, and a list of `i8` would have to be packed first.
- **An overload of `dot` on `bytes`.** `bytes` has no element signedness, so `dot` would pick one silently; two
  names keep the reading explicit (AI understanding first).
- **`vpmaddubsw` for `i8` times `i8` (the llama.cpp trick with `abs` and `sign`).** It saturates a pair at 16 bits
  and is wrong for `-128`; the widening form is exact for every byte. AVX-VNNI (`vpdpbusd`) and ARM64 `sdot` are
  later dispatch levels behind the same methods (ADR 0005 allows runtime dispatch); the ARM64 kernel uses no
  optional extension.
- **Returning a `float` with block scales.** That is application code; the exact `int` composes with it.
