# 0607. The Windows target is refused by `ail build` until its runtime lands

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.3](../spec/23-native.md)
- **Builds on:** [ADR 0597](0597-pe-executables-for-windows.md), [ADR 0598](0598-windows-imports-and-system-calls.md), [ADR 0606](0606-win64-calls-to-system-dlls.md)

## Context

The Windows target now has a PE writer, Win64 calls and an entry that runs the
runtime's start-up and exit (`target()` is 3). The runtime's system-call
branches, `sys_write`, `sys_exit`, `sys_mmap` and the `os.ail` low-level calls,
do not have Windows arms yet. Until they do, a Windows build would fall
through to Linux-style syscall numbers and produce an executable that does the
wrong thing.

## Decision

1. `ail build --target x86_64-windows` refuses with
   `error[native.build.3]` and the text "the Windows runtime (ADR 0607) has not
   landed". No executable is written.
2. The guard is removed in the change that adds the Windows arms: `sys_write`,
   `sys_exit`, `sys_mmap` (WriteFile, ExitProcess, VirtualAlloc through
   `import_call`) and the `os_win` wrappers for the `os.ail` low-level calls.
3. `pe-mir-sample` is not guarded: it is a test entry point, not a build.

## Rejected alternatives

- **Build and let the program fail at run time.** Rejected: a silently broken
  `.exe` is worse than a refusal at build time.
- **Keep the generic "unknown target" message.** Rejected: it hides that the
  target exists and is pending.

## Consequences

- Windows executables cannot be built until the runtime lands. The Wine tests
  in `native_pe` run the MIR and PE pieces directly, not `ail build`.
