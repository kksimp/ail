# 0021. Emit AIL's own debug format plus DWARF and PDB

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 26 — Debugging/profiling; 24 — Cross-platform
- **Spec rules:** none
- **Resolves:** [OQ-019](../open-questions.md)

## Context

[ADR 0016](0016-optimized-and-observable-debugging.md) requires richer debug
information than standard formats can fully express ("recompute this value
from X and Y"). Existing tools on each OS read only their native formats:
DWARF (gdb, lldb, perf, Linux and macOS crash tools) and PDB/CodeView (Visual
Studio, WinDbg, Windows crash dumps, ETW/WPA and most Windows profilers).

## Options considered

### Option A — Own format + DWARF + PDB (chosen)
- **Pros:** Every OS's standard tools work, so Windows stays first-class.
  AIL's own tools get complete information.
- **Cons:** PDB is complex and only partly documented, though Microsoft has
  published reference code. Three formats to maintain.

### Option B — Own + DWARF now, PDB before 1.0
- **Pros:** Less early work.
- **Cons:** Windows debugging is second-class during development.

### Option C — Own + DWARF only (designer's initial leaning)
- **Pros:** Two formats.
- **Cons:** Windows permanently second-class for debugging, profiling and
  crash analysis. Violates the first-class-OS principle.

## Decision

The compiler emits three debug formats from one internal source of truth:
1. **AIL's native debug format**: complete, including value recoverability
   (ADR 0016), optimisation decisions (ADR 0015) and allocation-ladder
   placement (ADR 0012). It's consumed by AIL's debugger, profiler and AI API.
2. **DWARF** on Linux and macOS.
3. **PDB/CodeView** on Windows.

The standard formats carry as much of the native information as they can
express.

## Consequences

- One internal debug-info model, with three writers. Writer tests
  cross-check against the information the native format holds.
- Clang, gcc and MSVC output may be used as **differential-testing oracles**
  for format correctness only (ADR 0010).
- Tier 1 CI must exercise gdb, lldb and WinDbg against AIL binaries.

## Rejected alternatives

- **PDB later:** leaves Windows second-class during development.
- **No PDB:** leaves Windows permanently second-class.
