# 0950. C ABI exports: `pub export c fn` and `ail header`

- **Status:** Accepted (front end and `ail header` built, `ffi.export.1-3`; the shared-library output is deferred past 0.1.0)
- **Date:** 2026-11-06
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 14 — FFI
- **Spec rules:** [ffi.export.1-3](../spec/14-ffi.md)
- **Builds on:** [ADR 0103](0103-c-header-import.md), [ADR 0432](0432-c-calls-through-two-register-files.md), [ADR 0820](0820-driver-level-unsafe.md)
- **Design:** [systems-abi.md](../design/systems-abi.md)

## Context

AIL can call C (`extern c fn`) but C cannot call AIL. To replace C, C++ and
Rust, AIL programs must be linkable libraries with a header C code can include.
The designer ruled out a no-GC mode and 32-bit targets (2026-11-05).

## Decision

1. `pub export c ["symbol"] fn` declares an exported function. The parser adds
   the attribute `export c` (a name no user can write), so the AST is unchanged.
2. The signature follows `ffi.decl.3` (rule `ffi.export.2`).
3. `ail header FILE [-o OUT.h]` generates the C header from the exports, so
   there is one source of truth (`ffi.export.3`).
4. Staged: this ADR's first slice is the front end and the header. The static
   library (`ail build --lib static`) follows, then shared libraries once the
   code generator can emit position-independent code. macOS and Linux first;
   Windows after its release gate.

## Rejected alternatives

- `@export` attribute: the same shape as `extern c fn` reads better, and `pub`
  already means "visible outside".
- Hand-written C headers: drift from the AIL source.
- Generating C and calling a C compiler: ADR 0010.
