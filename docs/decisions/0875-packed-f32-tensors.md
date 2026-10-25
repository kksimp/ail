# 0875. Native `Tensor[f32]` stores 4-byte floats

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24 03:38, "Push getting that 8 byte fix asap")
- **Builds on:** [ADR 0796](0796-tensor-f32.md), [ADR 0815](0815-native-tensors.md)
- **Topic:** 23 — Implementation (native tensors)

## Context

The first native tensors stored every element in an 8-byte slot, `Tensor[f32]` included.
That doubled the memory and bandwidth of f32 work and halved how many lanes a SIMD
register holds, against the reason `Tensor[f32]` exists (ADR 0796).

## Decision

- A native `Tensor[f32]` holds its elements packed, 4 bytes each, in row-major order.
  `Tensor[float]` stays 8 bytes.
- Kernels read and write f32 directly; arithmetic is IEEE single precision, rounding
  each result to f32 as ADR 0796 already specifies, so results don't change.
- Conversions between f32 and float happen only at explicit conversions and at element
  reads into `float` values.

Landed in #742 (batch BP-1).

## Consequences

Half the memory for f32 tensors, twice the SIMD lanes per register (ADR 0876), and no
change to printed results.

## Rejected alternatives

- **Keep 8-byte slots and narrow on output:** doubles bandwidth for nothing.
- **Pack only large tensors:** two layouts for one type, and every kernel tests which.
