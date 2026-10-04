# 0104. Every call into C is unsafe; safety comes from AIL wrappers

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 14 — FFI; 15 — Unsafe
- **Spec rules:** [ffi.call.1](../spec/14-ffi.md)

## Options considered

- **Always unsafe, with safe AIL wrappers (chosen):** an exact, honest
  boundary.
- **Annotating C functions as safe:** less ceremony, but unchecked claims
  can silently break memory safety.

## Decision

Calling a C function (imported or declared) is an unsafe operation. Safe
interfaces are AIL functions that check the C function's preconditions and
contain the call in an `unsafe` block.

## Consequences

- The standard library's OS layer is written this way on macOS and Windows,
  which must call system libraries (toolchain-constraints.md).
