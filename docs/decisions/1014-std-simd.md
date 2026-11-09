# 1014. `std.simd`: explicit lane vectors, defined lane by lane

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library; 25 — Optimization
- **Spec rules:** std.simd.1-9 ([std-simd](../spec/std-simd.md))
- **Extends:** [ADR 0005](0005-x86-64-baseline-v3.md), [ADR 0006](0006-arm64-baseline-v8-2.md), [ADR 0246](0246-simd-vectorisation.md), [ADR 0942](0942-user-loop-vector-kernels.md)

## Context

The auto-vectoriser (ADRs 0246, 0942, 0945) covers reduction nests and escape
loops. Other hot loops (image filters, codecs, hashing, parsing) need the
programmer to say which lanes work together, as Rust's `std::arch` and C's
intrinsics do. The designer's rule: intrinsics only, no `asm fn`, arm64 and
x86-64 only (ADRs 0004, 0008).

## Decision

1. **Types.** `std.simd` (AIL, `std/simd/simd.ail`) has `F32x4 F64x2 I32x4
   I64x2` (128 bits) and `F32x8 F64x4 I32x8 I64x4` (256 bits): value structs of
   their lanes.
2. **Meaning first.** Every operation is defined lane by lane with the scalar
   operation of the lane type (std.simd.3): IEEE, no fused multiply-add, wrapping
   integers, a fixed `min`/`max` for NaN and signed zero (std.simd.5), a
   balanced-tree `sum` (std.simd.8), masks as integer vectors (std.simd.7).
   Results are identical on every target and either width (ADR 0239's rule for
   reductions). A 256-bit type on a target without 256-bit registers is two
   128-bit halves; this is an implementation detail, not an API.
3. **Stage 1 (this change).** The module is plain AIL, generated from one
   template (`tools/simd/gen.py`) so the eight types cannot drift. It runs
   everywhere today and is the reference for stage 2. It adds no compiler
   feature, no runtime slot and no `unsafe`.
4. **Stage 2 (next slice).** Lower the lane structs of `std.simd` to vector
   registers: an SLP-style packing pass over the MIR of the module's inlined
   operations, with the instructions in `compiler/x64/kernels.ail` and
   `compiler/a64/kernels.ail` style (AVX2 on the x86-64-v3 baseline, NEON on
   ARMv8.2-A; `AIL_OPT_OFF=simd` keeps stage 1). Runtime dispatch to a wider unit
   (ADR 0005) is then an internal choice that must keep std.simd.3. The
   kernels are tested against stage 1 by a differential case.

## Rationale

Criterion 2 (AI understanding): lane-by-lane definitions with no
target-dependent result mean an agent never reads an ISA manual, and a test on
one machine is a test on all. Criterion 4 (speed) is served by stage 2 without
an API change. Stage 1 first because the vector-register work needs the Mac
and Linux gates and must have a reference to be tested against.

## Rejected alternatives

- **Raw intrinsics named after instructions (`_mm256_add_ps`).** One name per
  ISA instruction ties programs to a target, doubles the surface for the
  other ISA, and gives AI a table to memorise (criterion 2).
- **`asm fn`.** Refused by the designer (2026-11-09).
- **A generic `Vec[T, N]`.** Needs const generics in a std module (ADR 0954);
  eight named types read the same and check faster.
- **`fma`, integer `mul` for `I64`, saturating and widening ops, gathers,
  shuffles, 512-bit types.** Not in stage 1: each needs a scalar twin to define
  it by (`fma`) or has no instruction on both baselines (`I64` `mul`). Add them
  with their stage-2 kernels.
- **Platform-defined `min`/`max`.** Makes results differ between ARM64 and
  x86-64 (std.simd.5).

## Measurements

None for stage 1 (it is the scalar reference). Stage 2 reports speed against
stage 1 on `tools/perfbench` programs.
