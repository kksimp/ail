# 0608. The Windows x86-64 target: `target()` is 3, the entry runs the runtime, and output, exit and the heap use kernel32

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.4](../spec/23-native.md)
- **Builds on:** [ADR 0597](0597-pe-executables-for-windows.md) (PE writer), [ADR 0598](0598-windows-imports-and-system-calls.md) (imports), [ADR 0606](0606-win64-calls-to-system-dlls.md) (Win64 calls), [ADR 0607](0607-windows-runtime-guard.md) (the build guard this replaces)

## Context

Windows has no stable system calls, so the runtime reaches the OS through
kernel32 imports. The runtime picks its branch on `target()`, which the x86-64
backend fixes at build time.

## Decision

1. **`target()` is 3 for the Windows build** (x64 `prim` "target"), and 0 for
   Linux. The backend passes this through `gen_func`'s `win` flag.
2. **The Windows entry** (`generate_windows`) calls `rt.init(0, 0)` before
   `main` and `rt.exit(0)` after it, when the runtime is in the module.
   `rt.exit` flushes the output buffer and ends in `ExitProcess`.
3. **Runtime arms (`compiler/rt/native/core.ail`).** `sys_write` on target 3
   is `win_write`: `GetStdHandle` (-11 for standard output, -12 for standard
   error), then `WriteFile` in a loop. The byte count comes back through
   global slot `WIN_WRITTEN` (9, outside the ranges native-abi.md §6 reserves).
   `sys_exit` is `ExitProcess`. `sys_mmap` is `VirtualAlloc` with
   MEM_COMMIT | MEM_RESERVE and PAGE_READWRITE.
4. **macOS-only names stay out of the Windows table.** A libSystem call (a name
   starting with `_`, such as the ulock calls under `target() == 2`) becomes a
   trap on Windows, because kernel32 does not export it.
5. **The guard is removed.** `ail build --target x86_64-windows` builds the
   executable. ADR 0607's refusal no longer applies.

## Checks

- `examples/hello.ail` builds for `x86_64-windows`. Under Wine it prints
  `Hello, world!` and exits 0. Its import table lists four kernel32 functions:
  `ExitProcess`, `GetStdHandle`, `WriteFile` and `VirtualAlloc`.

## Rejected alternatives

- **Linking against the C runtime for output and heap.** Rejected: the
  runtime is AIL, and kernel32 covers these calls.
- **Importing every libSystem name on Windows, unresolved.** Rejected: a
  loader refuses an executable that imports a name its DLL does not export.

## Consequences

- Windows programs run only the runtime paths above so far. File-system,
  environment and time calls, and threads, have no Windows arms yet.
