# Source debugging and profiling (plan)

Non-normative. Staging for ADR 0016 and ADR 0021 (decided by the designer);
the stage ADRs are numbered by the lead.

## Stage 1: DWARF 4 for ELF (`debug.dwarf.1`)

`compiler/obj/dwarf.ail` adds `.debug_abbrev`, `.debug_info` and
`.debug_line` after the symbol table in every ELF executable (they are not
loaded, so they cost no memory at run time). The line table comes from the call
table the code generators already keep (ADR 0641): a row per user call at the
call's last byte, so a frame's `pc - 1` lookup finds it. The compile unit
carries one subprogram per function (`low_pc`, `high_pc`). gdb and lldb get
`break NAME`, `break FILE:LINE` on call lines, and backtraces with
`FILE:LINE`.

Rows also exist for checked operations (bounds, overflow, division). Known limits, each a later stage: other statements have no row; no variables;
no `.debug_frame` (the debuggers use the frame pointer every function keeps);
no Mach-O or PE output. Language is DW_LANG_C99, the closest one debuggers
evaluate expressions in.

## Stage 2: statement lines and variables

Record a row per MIR instruction that has a source position, and for each
source-named value (`Func.names`) its register or spill slot over its live
range (`DW_AT_location` lists). A value dropped by an optimisation records why
(ADR 0016). Mach-O: put the sections in a `__DWARF` segment.

## Stage 3: PDB / CodeView for Windows (ADR 0021)

## Profiler: `ail profile`

A timer signal (`SIGPROF`, `setitimer`) whose handler walks the frame pointers
into a preallocated buffer (no allocation in the handler), so the cost is
nil when off. `AIL_PROFILE=FILE` turns it on for any binary, `ail profile
PROG` is the wrapper. The samples map through `trace.tab`
(function, file, line). Output is JSON first (functions by self and inclusive
samples, hot lines, callers), then a text table for humans.
Linux needs a `sigreturn` restorer stub (a kernel in `x64/kernels.ail` and
`a64/kernels.ail`); macOS uses libSystem `sigaction`. Multi-thread: one timer
per thread (`timer_create` with `SIGEV_THREAD_ID`), a buffer per thread.

Rejected: instrumenting every function entry (a cost on the fast path, against
the goal of beating C); shelling out to `perf` or `sample` (not on every OS,
no JSON, no AIL names).
