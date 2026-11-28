# 1325. `bytes` reads of half-precision and bfloat16 numbers

- **Status:** Accepted
- **Date:** 2026-11-28
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Std; Language models
- **Spec rules:** type.bytes.20 ([04-types.md](../spec/04-types.md))
- **Extends:** ADR 1215

## Context

A pure-AIL language-model runtime (kksimp/Ail-LLM) reads GGUF and safetensors files through `fs.map`. Every GGUF block (Q8_0, Q4_0, the K-quants, F16 tensors) carries a half-precision scale, and bf16 weights and scales are common. Without a reader, a program decodes them through a 65,536-entry `List[float]` (512 KB, which evicts the cache in a hot kernel) or by bit arithmetic on every block.

## Decision

`bytes` gets `f16_le`, `f16_be`, `bf16_le` and `bf16_be`, which return a `float` widened exactly, with the bounds rule of `type.bytes.15`. They are methods: no new syntax and no new type. `bf16` is one shift into the `f32` read; `f16` is a small runtime function over the bits. There are no pushes.

## Rejected alternatives

- **`f16` and `bf16` types:** a type-system design that ADR 0055 left open; needs the designer's approval and is not needed to read a file.
- **Pushes (`push_f16_le`):** rounding a `float` to half precision needs its own rounding rules; no program has asked.
- **Lookup tables in the program:** what Ail-LLM does today; slower and bigger than one conversion.
