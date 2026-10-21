# 0103. C libraries are used through header import (AIL's own C parser) or hand-written declarations

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 14 — FFI; 17 — C interoperability
- **Spec rules:** [ffi.decl.1–3](../spec/14-ffi.md)

## Options considered

- **Header import plus manual declarations (chosen):** no hand-copied
  signatures, real types and docs for the AI API (brief §17), and quick
  manual declarations when needed. Con: building a C header parser is a
  substantial project.
- **Manual only:** signatures drift and contain mistakes.
- **Header import only:** blocks quick one-off declarations.

## Decision

1. `use c "header.h" as name` parses the C header with AIL's own C header
   parser (ADR 0009, so no clang) and exposes its declarations as AIL
   declarations under `name`. The generated declarations are cached, and
   can be viewed through tooling.
2. `extern c fn name(params) Result` declares a single C function by hand.
3. C declarations use C-compatible types and fixed layouts (ADR 0013).

## Consequences

- **Constraint, restated:** system headers rely on compiler extensions and
  macros. The parser must handle real-world headers on all five targets.
  Clang may be used as a differential-testing oracle for layouts (ADR 0010).
- How header search paths and per-target headers are configured comes with
  the build system and cross-compilation (ADR 0031).

## Rejected alternatives

- Manual declarations only: signatures drift and contain mistakes, and the AI API (brief section 17) would lack real types and docs. Rejected.
- Header import only: blocks quick one-off declarations, so manual `extern c fn` stays.
