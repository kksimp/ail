# 0815. Native tensors: a runtime object and kernels written in AIL

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076); the integrator approved the plan and the int-overflow answer
- **Topic:** 23 — Native backend (`std.tensor`, lane 174)
- **Spec rules:** `std.tensor.1-21` (17-std)

## Context

`tensor.Tensor[T]` stops 29 test programs and four examples and benchmarks
(`tensor`, `tensor_ops`, `examples/matrix`, `examples/tensors`) from building
natively. The interpreter's tensor (bootstrap/src/tensor.rs) is a shape plus
one contiguous row-major buffer of `int`, `float` or `f32` elements.

## Decision

1. **Layout.** A tensor is a pointer to a heap object of kind 4 (every word is
   scanned): `+0` element kind (0 `int`, 1 `float`, 2 `f32`, later), `+8` rank,
   `+16` element count, `+24` pointer to the elements (8 bytes each; kind 11, so
   the collector never scans them), `+32` the axis sizes. A tensor is never
   changed in place, so copying a tensor value shares the object.
2. **Runtime in AIL.** The kernels are `compiler/rt/native/tensor.ail`, linked
   only when used. `int` kernels trap on overflow (`std.tensor.6`) the way
   `sum` does in the interpreter and #386, so they use checked arithmetic and
   are not written in the wrapping runtime mode.
3. **Bit-identical to the interpreter.** Float kernels follow `std.tensor.5`
   (16 partial sums, halving 8/4/2/1, no fused multiply-add) and
   `std.tensor.7` (`matmul` is the plain triple loop), in plain scalar code.
4. **Traps.** A kernel that can fail has a runtime helper that returns the
   message (a string object) or 0. The lowering traps at the call with
   runtime message kind 15 when it is not 0, so the text and position are the
   interpreter's. Kind 15 prints the message string it is given.
5. **SIMD and `f32`.** Later. The scalar order already gives the vector
   units' results, so AVX2 and NEON kernels are an optimisation to add after
   a perfbench shows the gap. `f32` adds element kind 2 and two conversion
   operations.
6. **Order of work**, one pull request each: layout, `from`, `to_list`,
   `shape`, `len`, `==`, text; elementwise operators and reductions;
   `matmul` and the activations; the remaining `tensor_*` programs; `f32`;
   SIMD and parallel `matmul`.

## Amendment, 2026-10-21: how `f32` landed

`Tensor[f32]` keeps the layout above: element kind 2, one word per element,
the word a `float` already rounded to `f32`. One new primitive, `f32_round`,
rounds a `float` to single precision and widens it back (two conversion
instructions on each architecture), and the kernels apply it after every
operation. This needs no scalar `f32` type, no packed layout and no new
arithmetic instructions, and gives the interpreter's values exactly (for
`+ - * /` the double-precision result rounded once is the correctly rounded
single-precision one). Measured on the NumPy-comparison programs, the scalar
native kernels were *slower* than the interpreter's AVX2 and threaded ones
(`f32_matmul` 0.23 s against 0.084 s, `f32_chain` 0.46 s against 0.33 s). The
AVX kernels of `compiler/x64/kernels.ail` (`matmul`'s inner loop and the
elementwise operators, general loops that any tensor program uses) bring them to
0.05 s and 0.26-0.30 s on x86-64, against 0.05 s and 0.34-0.42 s for `ail run`
and 0.10-0.15 s and 0.24-0.37 s for NumPy. A Mac run showed the first NEON kernels behind `ail run` on `f32_matmul`, `softmax_sum` and `sigmoid`, so `matmul` became a register-blocked row kernel (`rt.vec_mm`: eight columns of sums stay in registers across the inner loop, `f32` as four singles per register) and `exp` lost its tail argument and most of its branches, with the same results bit for bit. The NEON kernels of `compiler/a64/kernels.ail` are the ARM64 half (two doubles at a time, the same operations and roundings).

## Rejected alternatives

- **Tensors as lists of lists.** Copying and indexing costs, and the shape and
  the `std.tensor.5` order would live in generated code instead of one kernel.
- **Kernels in Rust or C linked into the executable.** ADR 0010 and the
  self-hosting goal (ADR 0022) rule out anything but AIL in the runtime.
- **SIMD first.** The vector encoders don't exist in the native assemblers
  yet, and correctness against the interpreter comes first.
