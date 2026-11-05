# 0947. Native: DWARF debug info in every ELF executable, stage 1

- **Status:** Accepted
- **Date:** 2026-11-05
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 26 — Debugging/profiling
- **Spec rules:** `debug.dwarf.1`
- **Builds on:** [ADR 0016](0016-optimized-and-observable-debugging.md), [ADR 0021](0021-debug-info-formats.md), [ADR 0641](0641-native-trap-stack-from-frame-pointers.md)

## Context

ADR 0021 requires DWARF on Linux and macOS and PDB on Windows. ELF files had
only a symbol table, so gdb and lldb showed addresses, not lines.

## Options considered

### Option A — DWARF 4 from the call table, always on, ELF first (chosen)
`.debug_abbrev`, `.debug_info` (a compile unit and a subprogram per function)
and `.debug_line` (a row per user call, from `Image.calls`) after the symbol
table. The sections are not loaded: no run-time cost. No new flag; the file
grows by a few bytes per call.
- **Pros:** one change point (`symbols` in `obj/elf.ail`); both ELF forms and
  both machines; an agent gets `file:line` backtraces with no rebuild.
- **Cons:** lines only at calls; no variables yet.

### Option B — an opt-in `-g`
- **Pros:** smaller default files.
- **Cons:** a flag through the build cache key and every caller, and an agent
  has to rebuild to debug. ADR 0641 already made the call table unconditional.

### Option C — statement lines and variables at once
- **Cons:** needs per-instruction positions and location lists in both
  backends; too large to land untested in one step.

## Decision

Option A. Later stages: statement lines and variable locations, Mach-O
(`__DWARF` segment), PDB. If file size matters, `-g` can strip, not add.

## Consequences

- No `.debug_frame`: debuggers unwind by the frame pointer.
- Language is DW_LANG_C99 so expressions evaluate.
- Plan: [debugging-profiling.md](../design/debugging-profiling.md).
