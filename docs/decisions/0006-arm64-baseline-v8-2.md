# 0006. ARM64 baseline is ARMv8.2-A

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 24 — Cross-platform ABI strategy; 11 — Concurrency; 25 — Optimization strategy
- **Spec rules:** [plat.isa.2](../spec/01-platforms.md)

## Context

ARM64 capability levels that matter to AIL:

| Level | Relevant additions | Representative hardware |
|---|---|---|
| ARMv8.0-A | AArch64, NEON (always present) | Cortex-A53/A72, Raspberry Pi 3/4, AWS Graviton 1 |
| ARMv8.1-A | **LSE atomics** (CAS, LDADD, SWP…) | — |
| ARMv8.2-A | FP16, optional dot-product, RAS | Neoverse N1 (Graviton 2, Ampere Altra), Cortex-A76 (Raspberry Pi 5), Windows on ARM hardware |
| ARMv8.4/8.5+ | more crypto/flags features, BTI/MTE (8.5) | Apple M1 and later |
| ARMv9-A | SVE2, newer security features | Graviton 4, recent Neoverse/Cortex; **not** Apple M1–M3 |

AIL's concurrency-heavy runtime (scheduler, channels, GC barriers) depends
heavily on atomic operation performance. Without LSE, atomics fall back to
LL/SC loops, which are slower and scale worse under contention.

## Options considered

### Option A — ARMv8.2-A (chosen)
- **Pros:** Guarantees LSE atomics and FP16. Covers every Apple Silicon Mac,
  Graviton 2+, Ampere, Windows-on-ARM devices, and Raspberry Pi 5.
- **Cons:** Excludes Raspberry Pi 3/4, Graviton 1, and older Cortex-A53/A72 boards.

### Option B — ARMv8.0-A
- **Pros:** Runs on every 64-bit ARM device.
- **Cons:** No guaranteed LSE, so atomics either use LL/SC loops or need
  runtime dispatch on every atomic.

### Option C — ARMv9-A
- **Pros:** SVE2 scalable vectors.
- **Cons:** Excludes Apple M1–M3 (which don't expose SVE), so it's a
  non-starter as a baseline.

## Decision

AIL's ARM64 targets assume **ARMv8.2-A**. Later features (SVE/SVE2, SME, MTE,
dot-product, etc.) are used only through runtime-dispatched paths or explicit
opt-in target flags.

## Rationale

LSE atomics directly benefit the runtime and concurrency model. The hardware
that is excluded is old or hobbyist-tier and declining.

## Consequences

- The runtime and codegen can rely on LSE atomics without checks.
- SVE/SVE2 must be handled as an optional, dispatched capability. This
  matters for the SIMD design.
- Raspberry Pi 4 users are unsupported. Revisit if embedded ARM becomes a goal.

## Rejected alternatives

- **v8.0:** the atomic performance penalty hurts the runtime's core.
- **v9:** excludes most Apple Silicon.
