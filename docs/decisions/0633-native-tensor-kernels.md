# 0633. Float tensors in native code: layout, kernels and summation order

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library; 23 — Compiler architecture
- **Spec rules:** [native.tensor.1](../spec/23-native.md), std.tensor.1-13
- **Builds on:** [ADR 0632](0632-native-vector-primitives.md), [ADR 0328](0328-std-tensor-cpu-api.md), [ADR 0392](0392-native-structs-and-methods.md) (struct objects)

## Context

`std.tensor` ran only in the interpreter. The brief: bring the `float`
core to `ail build`, vectorised, with results identical to `ail run`.

## Decision

1. **Layout.** A `Tensor[float]` is the struct `tensor.Tensor` (kind 4) of
   two slots: `shape List[int]` and `data List[float]`, row-major. The
   lowering declares the struct itself (`Lower.declare_tensor`), so the
   generic copy, equality and field machinery of ADR 0392 apply: `a == b` is
   the struct `eq` (shapes, then elements, a NaN unequal to itself, as in the
   interpreter), and reading a tensor from a place copies it (own.val.1).
2. **Kernels** are runtime functions in `compiler/rt/native/tensor.ail`, on
   raw pointers, using the primitives of ADR 0632. Each computes what
   `bootstrap/src/tensor.rs` computes, in the same order:
   - `sum`, `dot`, `mean`: sixteen partial sums, element `i` in sum
     `i % 16`, halved (`acc[i] += acc[i + half]` for 8, 4, 2, 1), then the
     tail added in order (std.tensor.5).
   - `matmul`: each result row starts at 0.0 and adds `a[i][p] * b[p]`
     rowwise in order of `p`, the product rounded first (`v4axpy`).
   - elementwise operators: lane by lane; scalar operands broadcast
     (`v4ops`/`v4sop`); other shapes broadcast by materialising the
     stretched operands (`t_expand`), as the interpreter does.
   - axis `sum` and `mean`: contiguous axes through `sum`; strided axes
     add sixteen partial rows a whole row at a time (std.tensor.11).
     `max` keeps the first element; a NaN never replaces it.
   - `relu`, `neg`, `transpose`, `reshape`, `to_list`, `shape`, `len`.
3. **The interpreter's order decides.** Native and interpreter agree on
   every program of `tensor_kernels.ail` (12 lengths around the 4- and
   16-element block edges, 45 matmul shapes, 3-D broadcasting), on x86-64 and
   on ARM64 under qemu.
4. **Text** (`print`): `rt.out_tensor` writes `tensor.from([...], [shape])`
   as std.tensor.9 does.

## Measurements (Linux x86-64, `-mavx2`, min of 5)

| Program | native AIL | `gcc -O2 -mavx2` | interpreter | CPython (no NumPy) |
|---|---|---|---|---|
| 1M-element `a * b`, `sum`, `dot`, 20 rounds | 0.121 s (0.07 s page faults) | 0.046 s | 0.377 s | 2.09 s |
| 256x256 `matmul`, 10 rounds | 0.048 s | 0.037 s | 0.044 s | ~13.5 s (2 rounds: 2.71 s) |

The native elementwise case pays for fresh memory: the allocator never
frees until the collector lands (lane 171), so each round maps 24 MB.

## Alternatives rejected

- **A tensor as a flat `List[float]` plus a shape in a side table.** No
  place for the shape; breaks copy and equality.
- **Threaded `matmul` as the interpreter does for 64^3 and up.** Waits on
  lane 178's task runtime; a follow-up.
- **FMA kernels** (ADR 0632).

## Consequences

`int` tensors, `map`, indexing, `conv2d`, `argmax`, `clamp`, `round` and
`sigmoid`/`tanh` are reported by native.subset.1 until a later slice
(ADR 0635).
