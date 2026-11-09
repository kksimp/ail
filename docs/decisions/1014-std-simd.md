# 1014. `std.simd`: vector values in the backend

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library; 25 — Optimization
- **Spec rules:** std.simd.1-5 ([std-simd](../spec/std-simd.md))
- **Extends:** [ADR 0005](0005-x86-64-baseline-v3.md), [ADR 0006](0006-arm64-baseline-v8-2.md), [ADR 0246](0246-simd-vectorisation.md), [ADR 0942](0942-user-loop-vector-kernels.md)

## Context

The auto-vectoriser (ADRs 0246, 0942, 0945) calls hand-assembled `rt.vec_*`
kernels. Other hot loops need the programmer to say which lanes work together,
as Rust's `std::arch` and C's intrinsics do. The designer's rules: intrinsics
only, no `asm fn`, no ISA names in user code, arm64 and x86-64 only, and
results bit-identical on both.

## Decision

1. **Surface.** Fixed named value types, no generics: `f64x4`, `f32x8`,
   `i64x4`, `i32x8`, `u8x32` and the 128-bit forms. Operators work lane by lane;
   float operations are IEEE per lane and never fused; integer `+ - *` trap on
   overflow in any lane with `wrapping_*` as the escape; `sum()` uses the fixed
   tree of ADR 0239. Constructors and memory access are `simd.splat_T`,
   `simd.load_T` and `simd.store_T`. Comparisons give masks, with `select`.
2. **Backend.** MIR gets `Ty.Vec(kind)` and the code generators gain vector
   instructions in `compiler/x64` (VEX, ymm) and `compiler/a64` (NEON, a 256-bit
   value is a pair of q registers). The lowering is `compiler/mir/lower_simd.ail`
   (ADR 0823).
3. **S1 keeps vectors in memory.** A `Vec` value lives in four consecutive
   8-byte stack slots; each operation loads from the slots into the scratch
   registers (ymm14 and ymm15, or v30 and v31), operates and stores to the
   result's slots. No register class, no call clobbers and no callee-saved
   saves are needed, and no frame alignment (the accesses are unaligned). A cycle
   of vector moves at a block edge turns through four spare slots at the end of
   the frame. A vector is not a parameter, result, field, list element or call
   operand (`verify.ail`, std.simd.5).
4. **S1b (next, mandatory).** A vector register class (ymm0-ymm13; v0-v29 as q
   pairs) so values stay in registers, then spills and call clobbers (every
   vector register is call-clobbered; the prologue saves xmm6-xmm15 on Windows
   and d8-d15 on ARM64 only if used; `vzeroupper` before C calls). The MIR and
   the user surface do not change.
5. **Slices.** S1 (this change): `f64x4`, `splat`, `load`, `store`, `+ - * /`
   and `sum`. S2: the other floats, masks, `select`, `min`, `max`, `sqrt`,
   `lane`, `with_lane`. S3: integers. S4: bytes (`u8x32`) with a perfbench
   search against C. S5 (ADR 1030): the auto-vectoriser emits `Ty.Vec` and the
   hand-assembled kernels become AIL.

## Rationale

Criterion 4 (speed): one 256-bit load, operate and store replaces four scalar
operations, and S1b removes the memory round trip. Criterion 2: a lane-by-lane
meaning that is the same on both ISAs means no ISA manual is needed. Memory
residence first because the register allocator and the call conventions
are shared by every program, so a bug there breaks all of them, while a
vector kept in slots can only break programs that use vectors.

## Rejected alternatives

- **More hand-assembled kernels.** Each new loop shape needs two assembly
  routines; vector values in MIR serve every shape and the auto-vectoriser.
- **Raw instruction names (`_mm256_add_ps`).** Ties programs to an ISA and
  doubles the surface.
- **`asm fn`.** Refused by the designer.
- **A struct of lanes in AIL (the first try of this ADR, reverted).** Correct
  everywhere but every value stays scalar, so it is slower than the code it
  replaces, and `+` can't be defined on a struct (`trait.op.1`).
- **Methods `v.store(mut xs, i)` in S1.** The checker has no built-in method with
  a `mut` argument other than `self`; the function form `simd.store_f64x4` is used
  until it does.

## Measurements

None yet. S1b reports `tools/perfbench` programs against C.
