# 0597. Windows x86-64 executables: PE32+ written by the backend, with a fixed base and no relocations

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.build.4](../spec/23-native.md)
- **Builds on:** [AGENTS.md](../../AGENTS.md) (Windows is first-class), [ADR 0387](0387-static-executables-and-ail-build.md) (static ELF), [ADR 0389](0389-mach-o-signed-by-the-backend.md) (Mach-O, written by the backend), [ADR 0598](0598-windows-imports-and-system-calls.md) (imports)

## Context

Windows has no static executable format that avoids the loader. A PE image
always goes through the loader, which maps sections, resolves imports and
applies relocations. The native backend already writes its own ELF and
Mach-O files, so the same approach applies: the backend writes the PE, and
no linker is involved (ADR 0010).

## Decision

1. `compiler/obj/pe.ail` writes a PE32+ image for x86-64 (machine 0x8664),
   with the DOS header and stub, the PE signature, the COFF header, the
   optional header with its 16 data directories, and four sections:
   - `.text` (code and the entry point, read and execute);
   - `.rdata` (the read-only data: strings and float constants);
   - `.idata` (the import directory and address tables, read and write, since
     the loader fills the address tables in);
   - `.bss` (the runtime's globals, zero-filled, with no file bytes).
2. The image base is 0x140000000, the default for 64-bit executables. The
   section alignment is 4 KiB and the file alignment 512 bytes. The image has
   no relocation table and sets RELOCS_STRIPPED, so it loads only at its
   base. Position independence is not needed: every reference in the code is
   RIP-relative (ADR 0385).
3. The subsystem is CONSOLE. NX_COMPAT and TERMINAL_SERVER_AWARE are set.
   The stack reserve is 1 MiB and the commit 4 KiB, the usual defaults.
4. Relocations are `rel32` only. Symbols resolve as RVAs: `imp:NAME` to the
   import's address-table slot, `d:` to `.rdata`, `globals` to `.bss`, and a
   function name to `.text`. Each `rel32` is a displacement from the end of
   its instruction, as in the ELF writer.
5. Checked: the sample program (`ail run compiler pe-sample sample`) decodes
   with `objdump -x` and `llvm-readobj --coff-imports`, which read the same
   sections, entry point and imports. `bootstrap/tests/native_pe.rs` checks
   the structure without a Windows machine. The designer runs the sample on
   Windows when convenient (windows-status.md).

## Rejected alternatives

- **A linker (`lld`, `link.exe`).** Rejected: no external backend or linker
  is allowed (ADR 0010).
- **A relocatable image (`IMAGE_FILE_DLL` with `.reloc`).** Rejected for now:
  the code is position-independent already, but the loader's fixups add a
  table for no benefit while the image has one base. It can be added later
  without changing the code.
- **Static linking of the C runtime (`ucrt`).** Rejected: the runtime is
  written in AIL and needs only kernel32 (ADR 0598).
- **Writing the image with the kernel's own loader format through `ntdll`
  directly.** Rejected: not a stable interface, and it duplicates what the
  PE loader does.

## Consequences

- `ail build --target x86_64-windows` is not selected yet. It needs the
  Windows system-call shim (ADR 0598) and the Win64 calling convention at
  import call sites, which lane 120's codegen emits.
- The executable is unsigned. Windows runs unsigned console programs.
- If a later lane adds the optional-header `CheckSum`, the writer must update it.
