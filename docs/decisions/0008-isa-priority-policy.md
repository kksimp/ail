# 0008. ISA priority policy: ARM64 and x86-64 in strict lockstep

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Resolves:** OQ-001 (guardrails confirmed by ADRs [0025](0025-memory-model-atomics.md) and [0026](0026-hardware-feature-acceleration.md); CI by [0027](0027-ci-gating-all-targets.md))
- **Topic:** 24 — Cross-platform ABI strategy; 23 — Compiler architecture
- **Spec rules:** none yet

## Context

macOS is ARM64 only (ADR 0004). The designer considered dropping x86-64 on
Linux and Windows too, but was worried about adoption. Their current leaning
is **equal priority from day one**, with a worry that x86 might hold the
project back.

### Arguments for ARM64 leading
1. **Backend simplicity.** AArch64 has fixed 4-byte encoding, 31 general
   registers and three-operand instructions. x86-64 has variable-length
   encoding, prefix layers (REX/VEX/EVEX), 16 registers, and mostly
   destructive two-operand instructions. A first custom backend is faster to
   build and better on ARM64.
2. **The weak memory model finds bugs.** x86 is TSO (total store order) and
   hides missing barriers. ARM exposes them. A runtime or GC proven on ARM is
   correct on x86, but the reverse is not guaranteed.
3. **Trend.** All Macs, a growing share of cloud capacity (Graviton, Cobalt,
   Axion, Ampere), and Windows-on-ARM laptops.
4. **SIMD.** NEON is universal on ARM64, whereas x86 SIMD is fragmented
   (SSE/AVX2/AVX-512).

### Arguments against dropping x86-64
1. Most Windows machines are x86-64. Dropping it makes Windows second-class
   and breaks the founding principle that all three OSes are first-class.
2. Most Linux servers, CI runners and developer desktops are still x86-64.
3. The compiler itself must run on x86 hosts.
4. During bootstrap, x86-64 is nearly free through the temporary backend. The
   cost is only in the custom backend stage.
5. Having two ISAs keeps the IR and runtime honest. A single-ISA codebase
   accumulates hidden assumptions.

### Where x86 could actually hold AIL back, and the guardrails
| Risk | Guardrail (to be confirmed) |
|---|---|
| Language memory model shaped by x86 TSO | Define it on an abstract weakly-ordered machine. ARM64 gates concurrency/GC tests. |
| GC/runtime unable to use ARM's top-byte ignore (TBI), memory tagging (MTE) or pointer authentication (PAC) | Portable core, with hardware features as optional accelerators |
| SIMD API reduced to the lowest common denominator | Portable SIMD core plus first-class target-specific modules |
| Doubled backend effort | Shared machine-IR framework. Only instruction selection, encoding and ABI differ per ISA. |

Conclusion of the analysis: the language's syntax and type system are
independent of ISA. With these guardrails, x86 costs **engineering time**,
not **language quality**.

## Options considered

### Option A — Equal Tier 1, with ARM64 as the reference platform (architect's recommendation)
- **Pros:** No adoption hit. Weak-memory bugs are caught. The guardrails stop
  x86 from constraining the language. Stage milestones require parity.
- **Cons:** The most total work. x86 codegen may lag by weeks within a stage.

### Option B — Strict lockstep (chosen)
- **Pros:** Perfect parity at all times, and an IR that is forced to be neutral.
- **Cons:** The slowest iteration. x86 encoding complexity sets the pace for
  everything.

### Option C — ARM64 first, x86-64 by 1.0
- **Pros:** The fastest route to a high-quality first custom backend.
- **Cons:** A long x86 lag, and the risk that ARM assumptions creep into the IR.

### Option D — ARM64-only custom backend, x86 permanently on LLVM
- **Pros:** The smallest custom-backend scope.
- **Cons:** Breaks the "not permanently dependent on LLVM" goal, and gives the
  two ISAs different perf-report and debug behaviour.

### Option E — ARM64 only everywhere
- **Pros:** The smallest matrix.
- **Cons:** Excludes most Windows users, servers and CI. This would very
  likely be fatal to adoption.

## Decision

x86-64 and ARM64 are developed in **strict lockstep**. No change merges unless
it works on, and is optimised for, both ISAs on every OS where each applies.
Neither ISA is "primary".

## Rationale

The designer prioritises guaranteed parity and an IR forced to stay neutral
over iteration speed. The guardrails analysed above are adopted as separate
decisions: an abstract weak memory model (ADR 0025), portable runtime
features with optional hardware accelerators (ADR 0026), and all-target CI
gating (ADR 0027). Together they stop x86 from constraining the language,
while lockstep stops either ISA from lagging.

## Consequences

- Every backend feature (instruction selection, register allocation, ABI,
  SIMD lowering, debug and unwind info) is designed and landed for both ISAs
  together.
- x86-64's encoding complexity sets the pace of backend work. That cost is
  accepted.
- Weak-memory bugs are still caught, because ARM64 runs in the same CI gate
  as x86-64 (ADR 0027).
- Performance regressions on either ISA block merges. The benchmark suite
  runs on both.

## Rejected alternatives

- **Equal Tier 1 with ARM64 leading:** allows temporary drift. The designer
  prefers no drift at all.
- **ARM64-first, x86 by 1.0:** a long lag, and the risk of ARM assumptions in the IR.
- **ARM64-only custom backend with x86 on LLVM:** banned by ADR 0010.
- **ARM64 only:** fatal to adoption, and breaks first-class Windows and Linux.
