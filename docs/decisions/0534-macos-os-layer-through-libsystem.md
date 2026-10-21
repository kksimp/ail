# 0534. The macOS OS layer calls libSystem, not raw syscalls

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** Native runtime and ABI
- **Amends:** [ADR 0406](0406-raw-syscalls-for-native-runtime.md) (its macOS part), [ADR 0389](0389-mach-o-signed-by-the-backend.md) (imports)
- **Design note:** [native-abi.md](../design/native-abi.md) §4

## Context

ADR 0406 made the native runtime call the kernel directly on every target,
with libSystem as the macOS fallback. Apple documents only libSystem as
stable; raw syscall numbers and conventions have changed between macOS
releases, which is why Go moved to libSystem on macOS in 1.11. Two things
now make the fallback cheap: lane 178 (native tasks, ADR 0599) needs
`pthread_create` from libSystem and is adding dyld imports (a bind table and
GOT slots) to `compiler/obj/macho.ail`, and lane 177's OS layer is growing
from four shim calls to the whole file, process and clock surface, so every
raw call added now is one more that can break on an OS update.

## Decision

1. **On macOS the OS layer calls libSystem functions** (`_write`, `_read`,
   `_mmap`, `_munmap`, `_exit`, `_open`, `_close`, ...), through the dyld
   imports lane 178 adds, once they land. No raw `svc #0x80` remains in
   macOS code after the switch.
2. **Linux keeps raw syscalls.** The Linux syscall ABI is the kernel's
   documented stable interface, and static ELF executables need no loader.
3. Errors keep the runtime's convention, a negative errno: a shim returns
   `-(*__error())` when the libSystem call returns -1.
4. Imported calls use the C convention already used between AIL functions
   (native-abi.md §1); the caller treats caller-saved registers as
   clobbered, as for `extern` calls.
5. Until the bind table merges, the raw calls stay; the switch is one
   change to the macOS file of the OS layer (lane 177) and needs the Mac
   gate.

## Consequences

- macOS executables import from libSystem, which dyld loads for every
  executable anyway, so start-up cost is unchanged apart from binding a few
  symbols.
- Each OS call costs one indirect branch through a GOT slot more than a raw
  `svc`, which is small next to the kernel entry.
- Windows will follow the same rule (kernel32/ntdll, never raw syscalls)
  when it is built (`plat.win.1`).

## Rejected alternatives

- **Keep raw syscalls on macOS.** Works on today's macOS, but Apple can
  change numbers or calling details in any release and has done so; a
  shipped AIL binary would break with no rebuild possible for the user.
- **Raw syscalls with a libSystem fallback chosen at run time.** Two code
  paths to test, and the raw path is the one that silently breaks.
- **libSystem on Linux too (glibc or musl).** Needs a dynamic loader or a
  linked C library, against ADR 0387's static executables and ADR 0010's
  no-external-toolchain rule.
