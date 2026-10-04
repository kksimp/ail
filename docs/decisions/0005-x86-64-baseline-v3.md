# 0005. x86-64 baseline is x86-64-v3, with runtime dispatch for higher levels

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 24 — Cross-platform ABI strategy; 25 — Optimization strategy
- **Spec rules:** [plat.isa.1](../spec/01-platforms.md)

## Context

x86-64 has standardized microarchitecture levels:

| Level | Adds (summary) | Hardware |
|---|---|---|
| v1 | SSE2 (the original AMD64) | all x86-64 |
| v2 | SSE3/SSSE3/SSE4.1/SSE4.2, POPCNT | ~2009+ |
| v3 | AVX, AVX2, FMA, BMI1/2, LZCNT, MOVBE, F16C | Intel Haswell (2013)+, AMD Excavator/Zen+ |
| v4 | AVX-512 (F/BW/CD/DQ/VL) | Intel server, some client parts, AMD Zen 4+ |

The baseline determines what the compiler may emit **without** runtime checks.
AIL aims for C-class performance, so the default code quality matters.

## Options considered

### Option A — v3 baseline + runtime dispatch for v4 and beyond (chosen)
- **Pros:** 256-bit SIMD, FMA, and BMI2 by default, which means much better
  autovectorization and bit-manipulation code. Major distributions are moving
  this way (RHEL 10 requires v3).
- **Cons:** Excludes pre-2013 Intel, some low-end Atom/Celeron/Pentium chips
  without AVX, and some VMs that mask AVX.

### Option B — v2
- **Pros:** Matches the effective Windows 11 (24H2) floor and runs almost everywhere.
- **Cons:** Only 128-bit SIMD unless dispatched, so the default code is weaker.

### Option C — v1
- **Pros:** Runs on every x86-64 CPU ever made.
- **Cons:** The weakest code, and it needs dispatch for any serious performance.

### Option D — v4
- **Pros:** The widest SIMD, plus mask registers.
- **Cons:** Missing on many consumer Intel chips (Alder Lake onward disabled
  AVX-512) and on older AMD. It excludes too many machines.

## Decision

AIL's x86-64 targets assume **x86-64-v3**. Features beyond v3 (e.g. AVX-512)
are used only through **runtime-dispatched** code paths or explicit opt-in
target flags.

## Rationale

The language's performance goals depend on good default vectorization. Hardware
older than v3 is more than 13 years old and shrinking.

## Consequences

- The compiler, the stdlib and the runtime can use AVX2/FMA/BMI2 freely on x86-64.
- A runtime CPU-feature-detection and dispatch mechanism is needed (a design
  item for the SIMD/stdlib topics).
- A binary run on hardware older than v3 must fail with a clear diagnostic,
  not a crash. See spec rule `plat.isa.3`.

## Rejected alternatives

- **v1/v2:** give up default performance for hardware that is mostly retired.
- **v4:** excludes a large share of current consumer CPUs.
