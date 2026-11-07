# 0980. Native C calls on Windows: `LoadLibraryA`, `GetProcAddress` and the Win64 convention

- **Status:** Accepted
- **Date:** 2026-11-07
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 14 — FFI; 23 — Compiler architecture
- **Spec rules:** [ffi.load.1](../spec/14-ffi.md), [ffi.load.3](../spec/14-ffi.md), [ffi.map.1](../spec/14-ffi.md)
- **Builds on:** [ADR 0870](0870-native-extern-c-bound-lazily-with-dlopen.md), [ADR 0606](0606-win64-calls-to-system-dlls.md)

## Context

Seed e's Windows check found `ffi_dynamic` and `ffi_f32` trapping with `ffi.load.3` ("not supported on Windows yet"). The lazy binding of ADR 0870 had no Windows half, and the indirect C call used the System V registers.

## Decision

1. **Binding.** On Windows `rt.ffi_addr` opens a library with `LoadLibraryA` and finds a symbol with `GetProcAddress`; both are kernel32 imports, so the PE import table gains no DLL. `ffi.load.1` and `ffi.load.2` trap as on the other targets.
2. **Library names.** A name with `/` or `.` is used as written. `c`, `libc`, `m`, `libm` and `msvcrt` name the Universal CRT, `ucrtbase.dll`. Any other name tries `NAME.dll`, then `libNAME.dll`. With no library the program's C functions are looked up in `ucrtbase.dll`, then `msvcrt.dll`.
3. **The call** (x86-64) follows Win64: the argument's position picks `rcx rdx r8 r9` or `xmm0`-`xmm3`, the caller reserves 32 bytes of shadow space, and later arguments, integer or float, go on the stack above it. `xmm6`-`xmm15` are callee-saved, so the `xmm8`-`xmm13` save of ADR 0870 is skipped. An `f32` stays in the low half of its register (ffi.map.1).
4. **ARM64.** Windows on ARM64 uses AAPCS64 for these calls, as the other ARM64 targets do, so its call sequence is unchanged and only the binding above applies. aarch64-windows stays experimental (ADR 0968): it must compile, and is unverified.
5. **`ffi.load.3`** keeps its ID. It now describes the library naming above, and a Windows bind failure traps with `ffi.load.1` or `ffi.load.2`.

## Rejected alternatives

- **Static imports per declared library:** a missing DLL would fail the whole program at load instead of trapping at the call (the reason of ADR 0870).
- **Linking `msvcrt.dll` for `m` and `c`:** older, and not shipped as the C runtime of current Windows.
- **Retiring `ffi.load.3`:** published rule IDs are never renumbered.

## Consequences

`ffi_dynamic` and `ffi_f32` must match their goldens on ail-win. A C library that needs another calling convention (`stdcall` on 32-bit) isn't supported, as there is no 32-bit target. Variadic C functions are still unsupported (Win64 variadics also need floats mirrored into integer registers).
