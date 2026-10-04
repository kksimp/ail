# 0097. A module is a directory of `.ail` files

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 12 — Modules
- **Spec rules:** [mod.unit.1–2](../spec/12-modules.md)
- **Background:** [design/concurrency-modules-options.md](../design/concurrency-modules-options.md)

## Options considered

- **Directory (chosen):** splitting or merging files never changes code, and
  there are fewer imports. Con: a module's names are spread across files
  (tools locate them).
- **File:** self-contained files, but moving code changes imports elsewhere.

## Decision

1. All `.ail` files directly inside one directory form one module. Every
   top-level name is visible in every file of the module, without imports.
2. A module is named by its directory. Its import path is the directory path
   relative to the package root, with `.` separators (ADR 0099).

## Consequences

- Duplicate top-level names across files of one module are a compile-time error.
- Subdirectories are separate modules.
