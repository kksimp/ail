# 0389. Mach-O executables, dynamically loaded but importing nothing, signed ad hoc by the backend

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.3](../spec/23-native.md)
- **Builds on:** [ADR 0387](0387-static-executables-and-ail-build.md) (static ELF), [ADR 0406](0406-raw-syscalls-for-native-runtime.md) (raw syscalls on macOS)

## Context

macOS has no static executables: the kernel hands every main executable to
dyld. On Apple Silicon an executable must also carry a code signature, at
least an ad hoc one. The brief allowed a documented `codesign -s -` step.

## Decision

1. `compiler/obj/macho.ail` writes an `MH_EXECUTE`, `MH_PIE` file with
   `__PAGEZERO`, `__TEXT` (`__text` code and `__const` data), a zero-filled
   `__DATA,__bss` for the runtime's globals, and `__LINKEDIT`. It loads
   `/usr/lib/dyld` and names `libSystem.B.dylib` (dyld requires that of a
   main executable) but imports no symbol: the runtime makes system calls
   itself (ADR 0406). `LC_MAIN` points at the entry. No fixups: all code is
   PC-relative.
2. **The backend signs the file itself.** It appends a superblob with one
   code directory (version 0x20400, `CS_ADHOC | CS_LINKER_SIGNED`,
   SHA-256 of each 4 KiB page, the executable segment marked main binary),
   the same shape `ld64` and `lld` write. SHA-256 is AIL
   (`compiler/obj/sha256.ail`), checked against Python's `hashlib`.

## Options considered

- **Run `/usr/bin/codesign -s -` from the driver.** Rejected: a dependency
  on a host tool for every build, and Mach-O files couldn't be built on
  Linux.
- **Static Mach-O without dyld.** The kernel refuses it on current macOS.
- **Import `write`, `mmap` and `exit` from libSystem.** The fallback of
  ADR 0406 if raw syscalls break; it adds binds and stubs. Not needed yet.

## Consequences

- `ail build --target aarch64-macos` works from any host, including Linux.
- The signature is ad hoc: Gatekeeper treats downloaded binaries as
  unsigned, which is expected for locally built tools.

## Rejected alternatives

- Running `/usr/bin/codesign -s -` from the driver: rejected because every build would depend on a host tool, and Mach-O files could not be built on Linux.
- A static Mach-O without dyld: rejected because the kernel refuses it on current macOS.
- Importing `write`, `mmap` and `exit` from libSystem: kept as the fallback of ADR 0406 if raw syscalls break. It adds binds and stubs, which are not needed yet.
