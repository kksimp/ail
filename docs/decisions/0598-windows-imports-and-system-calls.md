# 0598. Windows imports: kernel32 through an import address table, and the Win64 convention at each call

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.4](../spec/23-native.md)
- **Builds on:** [ADR 0406](0406-raw-syscalls-for-native-runtime.md) (raw system calls on Linux and macOS), [ADR 0597](0597-pe-executables-for-windows.md) (the PE writer)

## Context

Linux system calls are a stable interface, and macOS binaries are allowed
to make them directly (ADR 0406). Windows is different: its system call
numbers change between releases, so the stable interface is kernel32 (and
the DLLs it forwards to). The native runtime's system call primitive has
no Windows meaning, so Windows needs a different path for the same
operations: standard handles, output, and process exit, at least.

## Decision

1. **Imports are kernel32 functions, bound by the loader.** The `.idata`
   section of ADR 0597 holds one descriptor per DLL and, for each function,
   a slot in the address table. The loader writes the function's address
   into the slot at load time. The first DLL is `KERNEL32.dll`; other DLLs
   are added the same way when a lane needs them (for example
   `bcryptprimitives.dll` for random bytes).
2. **A call to an import is an indirect call through its slot:**
   `call qword ptr [rip + slot]`, encoded FF 15 with a `rel32` relocation to
   the symbol `imp:NAME`. The codegen emits the instruction and the
   relocation, and `pe.ail` resolves it.
3. **The Win64 calling convention applies at each import call site only.**
   Arguments go in RCX, RDX, R8 and R9, then on the stack; the caller
   reserves 32 bytes of shadow space above the return address, keeps the
   stack 16-byte aligned at the call, and treats RAX, RCX, RDX, R8, R9, R10,
   R11 and XMM0-XMM5 as clobbered. Everything else stays on the System V
   layout the backend already uses. Each import call site moves its
   arguments into those registers.
4. **The runtime's system call primitive gets a Windows variant.** Where the
   Linux and macOS targets emit `syscall`, the Windows target emits a call
   to the import that does the same job: `GetStdHandle` and `WriteFile` for
   output, `ExitProcess` for exit, `VirtualAlloc` for the allocator's pages,
   `GetCommandLineW` for arguments. The primitive keeps its meaning, so the
   AIL runtime above it does not change. Each mapping is a case in lane 120's
   code generator, with a cited rule for each trap that Windows cannot
   provide.
5. **A bounded set of imports.** The first set is `GetStdHandle`,
   `WriteFile`, `ExitProcess`, `VirtualAlloc`, `VirtualFree`, and
   `GetCommandLineW`. Adding an import means adding its name to the
   program's `PeDll` list and to this ADR.

## Rejected alternatives

- **Raw Windows system calls (`syscall` with the service numbers).**
  Rejected: the numbers change between Windows releases, and Microsoft
  documents no stable interface for them.
- **Calling ntdll directly.** Rejected: also not a stable interface, and it
  is no simpler than kernel32 for these calls.
- **A C runtime (`msvcrt`, `ucrt`).** Rejected: the runtime is in AIL and
  needs only the calls above.
- **Delay-loaded imports.** Rejected for now: a short import list loads at
  start-up with no measurable cost.

## Consequences

- The runtime's Windows path is a small table in the code generator, not a
  change to the AIL runtime.
- The sample in `compiler/obj/pe.ail` (`pe-sample`) checks the call sites and
  the import table. It is not a substitute for running a program on Windows.
- Lane 120 must emit the Win64 argument moves and shadow space at import
  call sites. Until it does, the Windows target does not build programs.
