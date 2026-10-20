# 0436. Native C calls use the platform convention and dynamic imports, planned before built

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 14 — FFI; 23 — Compiler architecture
- **Spec rules:** [ffi.native.1](../spec/14-ffi.md), [ffi.link.1](../spec/14-ffi.md)
- **Builds:** [ADR 0105](0105-libc-only-with-c.md); coordinates with lane 120 (native backend) and lane 122 (runtime) through `docs/design/native-abi.md` §9

## Context

The native backend (ADRs 0384–0393) writes static executables with no libc.
ADR 0105 says a program that declares or imports no C functions must not depend
on libc, and one that does links the system libc. The interpreter's `extern c
fn` exists (ADRs 0431–0432); the compiled side needs an agreed shape before
lane 120 writes its object writers, so that the first dynamic `extern` does not
force a redesign of the ELF and Mach-O writers.

## Decision

1. A native call to C is the platform's register convention with no thunk. The
   checker's limits (six integer-like and eight float arguments, no variadics,
   no structs) are exactly what fits in registers, so there is no stack
   argument code to write.
2. A program with no `extern` stays a static executable. A program with one
   becomes a dynamic executable: `PT_INTERP` plus `DT_NEEDED` and
   `BIND_NOW` GOT slots on ELF; `LC_LOAD_DYLIB` and chained fixups on Mach-O.
   Lazy PLT stubs are not used (fewer moving parts, and a missing symbol fails
   at load rather than at the first call).
3. The runtime entry stays `rt_start` and does not call `__libc_start_main`;
   `rt_exit` flushes C stdio through `fflush(0)` when libc is linked. This is
   recorded as an open risk with a named test (`printf` on Linux ARM64 and
   macOS).
4. `mem.*` uses libc `malloc`/`free` when libc is linked so blocks cross the
   boundary, and the runtime's allocator otherwise.
5. Until the writers support it, `ail build` rejects `extern` and `std.mem` with
   `ffi.native.1`. Checking and the interpreter are unaffected.

## Options considered

### Option A — dynamic imports, platform convention (chosen)
- **Pros:** the only way to reach `libSystem` on macOS and libc on Linux
  without writing a libc; the declaration `extern c "lib" fn` already carries
  the library; the register shape is the same as the interpreter's, so one
  differential test covers both.
- **Cons:** two object-writer features (dynamic section, fixups) with real
  loader subtleties; entry without `__libc_start_main` is a risk.

### Option B — static linking of a libc
- **Cons:** no libc to link (ADR 0010, ADR 0017); on macOS static linking of
  `libSystem` is unsupported.

### Option C — raw system calls for everything
- **Cons:** cannot call arbitrary C libraries (`libm`, SQLite); macOS does not
  promise syscall numbers.

### Option D — enter through `__libc_start_main`
- **Pros:** libc is fully initialised.
- **Cons:** ties the entry sequence and stack setup to glibc and musl internals
  per target; revisit if Option A's risk shows up in the stdio test.

## Consequences
- Lane 120 adds the dynamic section/fixups when it is ready; lane 122 keeps
  `rt_exit` in one place so the flush is a three-line change.
- Tokens: no syntax change, so no tokbench table.
