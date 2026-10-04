# 0010. No LLVM (or other external) backend; existing compilers are for comparison only

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 23 — Compiler architecture; 25 — Optimization strategy
- **Spec rules:** none
- **Changes:** the brief's Stage 4 ("native compiler using an existing backend") and roadmap refinement #3

## Context

The founding brief allowed LLVM as a temporary bootstrap backend. The
architect recommended keeping it as a permanent benchmark reference, with the
custom backend becoming the default only once it reached parity. The designer
rejected using LLVM as a crutch in any form.

## Options considered

### Option A — Custom backend from the start; LLVM, GCC and others only for comparison (chosen)
- **Pros:** No architecture ever shaped by LLVM's IR or assumptions. All
  backend effort goes into AIL's own backend. Compliant with ADR 0009.
- **Cons:** First native binaries arrive later. Early native code will be
  slower than LLVM-generated code until the optimizer matures.

### Option B — LLVM as a temporary bootstrap backend (the brief's original plan)
- **Pros:** Fast route to optimised native code.
- **Cons:** Its IR and design leak into the compiler, and removing it later is
  costly. The designer rejected this.

### Option C — LLVM as the default until the custom backend is faster
- **Pros:** Users always get the best code.
- **Cons:** Effectively a permanent crutch. Rejected.

## Decision

AIL never ships code generated through LLVM or any other third-party backend,
including during bootstrap. Clang, GCC, rustc and similar compilers may be
used **only** to:
- produce comparison binaries for benchmarks, and
- serve as differential-testing oracles (e.g. checking that C FFI layouts
  match).

## Rationale

Owning the backend from day one keeps the IR, debug info, optimisation reports
and performance-transparency data entirely under AIL's control. The designer
accepts slower early code in exchange.

## Consequences

- Roadmap Stage 4 becomes "native compiler using AIL's own simple backend".
  The Stage 3 interpreter becomes the main tool for validating semantics.
- The custom backend has to be designed for incremental improvement: a simple,
  correct code generator first, then optimisation passes.
- Benchmarks compare AIL against clang/gcc -O2/-O3 and rustc from the first
  native binary, so the gap is always visible and published.
- Emitting C as an intermediate step would also count as an external backend.
  Whether that is allowed is tracked separately (OQ-012).

## Rejected alternatives

- **LLVM bootstrap:** its assumptions leak into the architecture.
- **LLVM until we're faster:** a permanent crutch in practice.
