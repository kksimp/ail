# 0606. Win64 calls to system DLL functions: `libsys.NAME` through the PE import table

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.4](../spec/23-native.md)
- **Builds on:** [ADR 0597](0597-pe-executables-for-windows.md) (PE writer), [ADR 0598](0598-windows-imports-and-system-calls.md) (imports and the Win64 convention), [ADR 0385](0385-x86-64-code-generation.md) (x86-64 code generation)

## Context

A runtime file calls a system DLL function with `import_call("Name", ...)`,
which lowers to a MIR `call` named `libsys.Name`. Windows needs those calls
to follow the Win64 convention and to reach the function through the import
table. Linux and macOS have no such calls, so the lowering is only on the
x86-64 path.

## Decision

1. **The MIR shape is unchanged.** A `libsys.NAME` call is an ordinary call:
   its arguments are value ids, its result is an integer or float as the
   signature says, and `site` is -1, so it gets no stack check (ADR 0535).
2. **Arguments go by position** in Win64: argument k in 0-3 goes to rcx, rdx,
   r8 or r9 if it is an integer, or to xmm0-3 if it is a float. Arguments
   from the fifth on go to `[rsp + 32 + 8*(k-4)]`.
3. **Stack room**: the caller subtracts `32 + 8*max(0, n-4)`, rounded up to
   16 bytes, before it places the arguments, and adds it back after the call.
   Frames keep rsp 16-byte aligned at call sites, so the callee sees the
   alignment Win64 requires. Slots are RBP-relative, so the adjustment does
   not move them.
4. **The call is indirect through the import slot**: FF 15 with a `rel32` to
   `imp:NAME` (ADR 0598). `compiler/obj/pe.ail` builds the table: `pe_image`
   collects the distinct names, groups each under its DLL (`pe_dll_of`), and
   writes one slot per name.
5. **The result** is rax, or xmm0 for a float.
6. **Clobbers**: every SysV caller-saved register is treated as clobbered. This
   is a superset of Win64's volatile set, so no extra saves are needed.
7. **DLL groups**: kernel32 is the default; sockets use WS2_32.dll; `ProcessPrng`
   uses bcryptprimitives.dll. Each DLL is one descriptor, in a fixed order.

## Rejected alternatives

- **A new MIR op for system calls.** Rejected: a runtime call is enough, and
  the lane rule keeps new ops out of the shared MIR (lane 170).
- **Passing arguments by the System V convention and adapting at the entry.**
  Rejected: Win64 positions and the shadow space differ at every call site, so
  the adaptation would need a stub per import.
- **Resolving imports at run time with `GetProcAddress`.** Rejected: the loader
  already fills the table, and run-time lookup adds a dependency on kernel32
  at a point where it is not yet loaded.

## Checks

- `ail run compiler pe-mir-sample x` builds a MIR module with one
  `libsys.GetStdHandle(-11)` call and returns 7. `x64::generate_windows` lowers
  it, and the PE runs under Wine (apt `wine`, 9.0) with exit status 7. That
  shows the call went through the import table and returned. It does not
  check the argument value.
- The `native_pe` tests run both checks when Wine is installed.

## Consequences

- `libsys.` calls on the x86-64 Linux target would reach an unresolved
  `imp:` symbol. Only the PE writer resolves them, so a program that uses
  them must target Windows.
- The lowering is checked by reading and by the PE structure tests. It is
  not run until a runtime file emits `libsys.` calls (lanes 177 and 178).
