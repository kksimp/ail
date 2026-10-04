# 0105. On Linux, libc is linked only when a program uses C code

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 14 — FFI; 22 — Runtime
- **Resolves:** OQ-021
- **Refines:** [ADR 0020](0020-linux-no-libc.md)

## Options considered

- **libc only when C is used (chosen):** pure-AIL programs keep ADR 0020's
  benefits, and C interop just works. Con: two Linux runtime modes to test.
- **AIL's own libc layer:** large and risky.
- **No C interop on Linux:** fails brief §17.

## Decision

1. A Linux program that doesn't use C libraries is libc-free (ADR 0020).
2. If any part of a Linux program imports or declares C functions, the
   toolchain links the system libc dynamically, and the runtime starts and
   manages threads in a libc-compatible way for that program.

## Consequences

- The Linux test matrix covers both modes.
- The performance report and build output say which mode a binary uses.
