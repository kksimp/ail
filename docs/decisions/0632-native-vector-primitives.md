# 0632. Vector primitives for the native runtime: AVX2 and NEON, in lockstep

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.simd.1](../spec/23-native.md), std.tensor.5
- **Builds on:** [ADR 0005](0005-x86-64-baseline-v3.md) (x86-64-v3), [ADR 0006](0006-arm64-baseline-v8-2.md) (ARMv8.2-A), [ADR 0008](0008-isa-priority-policy.md) (lockstep), [ADR 0387](0387-static-executables-and-ail-build.md) (primitives)

## Context

The designer wants modern CPUs used automatically. The interpreter's tensor
kernels already run on AVX2 or NEON (std.tensor.5), but the native runtime,
written in AIL, had no way to name a vector instruction: its only
primitives move one 8-byte word.

## Decision

1. **Six primitives**, declared in `compiler/rt/native/prim.ail` and
   expanded inline like `load64`: `v4op(op, dst, a, b)`, `v4ops(op, dst,
   a, s)`, `v4sop(op, dst, s, a)`, `v4axpy(c, x, b)`, `v16acc(acc, p)` and
   `v16dot(acc, p, q)`. They work on four doubles (or sixteen, against an
   accumulator block) at the addresses given, `op` a literal (0 add, 1
   subtract, 2 multiply, 3 divide).
2. **No new register classes.** The allocator still sees only scalar
   `float` values. The primitives read and write memory and use the two
   scratch vector registers (ymm14/15 on x86-64, v30/v31 on ARM64), so
   neither the allocator nor MIR gained a vector type and `lower.ail` only
   gained six names in `PRIMS`.
3. **Encoders** (`Asm.vmovupd_rm/mr`, `vpd256_rrr`, `vbroadcastsd` on
   x86-64, with the VEX `L` bit; `ldr_q`, `str_q`, `fop_2d`, `dup_2d`,
   `fmul_2d_lane` on ARM64), each checked against objdump or llvm-mc by
   `bootstrap/tests/native_asm.rs`.
4. **No fused multiply-add.** `v4axpy` and `v16dot` round the product, then
   add, as the interpreter's kernels do (std.tensor.5), so AVX2, NEON, the
   scalar fallback and the interpreter agree bit for bit. Accumulators live
   in memory between the vector operations: with sixteen partial sums and
   only two scratch registers, the loads and stores pipeline well enough
   (measured: matmul within 1.3x of `gcc -O2 -mavx2`).
5. **NEON works two doubles at a time**, so a four-double primitive is two
   `q` operations. Runtime dispatch to AVX-512 or SVE is out of scope
   (ADR 0005).

## Alternatives rejected

- **A vector `Ty` in MIR and the allocator.** Right for an auto-vectorising
  optimiser, but touches every pass and `lower.ail`; kernels written by
  hand in the runtime need none of it. Revisit with lane 191's optimiser.
- **Whole kernels hand-assembled in the code generators.** Fast, but the
  designer's direction is that the runtime is AIL; kernels in AIL can be read
  and tested under the interpreter's rules.
- **Fused multiply-add for `dot` and `matmul`.** About 1.3x faster on
  `matmul`, but rounds once instead of twice and would break the
  bit-identical results (std.tensor.5) that `.out` files rely on.

## Consequences

Any runtime code can use the SIMD primitives; tensors (ADR 0633) are the
first user. Windows x64 has the same encoders and the same code path.
