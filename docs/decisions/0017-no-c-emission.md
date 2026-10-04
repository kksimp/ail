# 0017. The compiler never emits C as a path to native code

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 23 — Compiler architecture
- **Spec rules:** none
- **Resolves:** [OQ-012](../open-questions.md)

## Context

[ADR 0010](0010-no-llvm-backend.md) bans LLVM and other external backends.
Emitting C and compiling it with clang or gcc would be another way to reach
native code early.

## Options considered

### Option A — No: own backend only (chosen)
- **Pros:** No external compiler influences the IR. No crutch.
- **Cons:** No native binaries until the custom backend works. The
  interpreter carries semantic testing until then.

### Option B — Yes, for internal tests only, never shipped
- **Pros:** Native-speed test runs sooner.
- **Cons:** Still a crutch. C undefined behaviour can contaminate results.
  Throwaway work.

## Decision

AIL never translates programs to C (or any other high-level language) as a
route to executable code. This applies to tests too.

## Rationale

It follows directly from ADR 0009 and ADR 0010.

## Consequences

- The Stage 3 interpreter is the only way to execute AIL before the native
  backend exists. Its role as the semantic **reference** grows (see ADR 0018).
- Generating C *headers* for FFI (so C code can call AIL) is a separate matter
  and is not banned by this ADR. That's an FFI-topic question.

## Rejected alternatives

- **C emission for tests:** a crutch, with undefined-behaviour contamination risk.
