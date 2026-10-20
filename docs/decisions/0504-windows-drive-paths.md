# 0504. Windows drive paths in std.path and std.fs.9

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard modules (Windows)
- **Spec rules:** [std.path.2](../spec/17-std.md), [std.path.4](../spec/17-std.md), [std.fs.9](../spec/17-std.md)
- **Criterion that decided:** Understanding (criterion 2). A drive root is a root on Windows; without it `dir` and `join` return wrong text, and an AI reading the result would build invalid paths.

## Context

The settled constraints (AGENTS.md) make Windows a first-class target. `std.path` (ADR 0253) and
`std.fs.glob` (ADR 0281) were written for `/` paths. On Windows, `path.dir("C:\x")`
returned `C:`, `path.join("C:\a", "C:\b")` returned `C:\a/C:\b`, and
`fs.glob("C:\src\*.ail")` matched nothing, because glob split only at `/`.

## Options considered

### Option A — keep path text as it is; document that Windows paths are unsupported
- **Pros:** no code change.
- **Cons:** breaks the first-class Windows promise; agents would receive wrong paths silently.

### Option B — treat a drive root as a root on Windows, and split globs at `\` there (chosen)
- **Pros:** the fix is local to two helper functions; POSIX behaviour is unchanged and is checked by the same unit tests.
- **Cons:** `join` still joins with `/` (spec std.path.4), so Windows results can mix separators. Windows accepts both.

### Option C — make `join` use `\` on Windows
- **Pros:** the result looks native.
- **Cons:** changes the spec text and every program that builds Windows paths with `/`. Costs understanding (criterion 2) for little gain, since Windows accepts `/`.

## Decision

Option B. `dir` keeps a drive's separator (`dir("C:\x")` is `C:\`). `join` treats
`C:\…` as absolute on Windows only. A glob pattern starting with `C:\` is
absolute, and glob components split at `\` on Windows. The helpers take the platform
as a parameter, so unit tests check Windows and POSIX rules on any host.

## Consequences

- `bootstrap/src/stdlib.rs` tests `path_tests`, and `bootstrap/src/interp/osext.rs` test `glob_roots_on_both_platforms`.
- No interpreter hot path changes. These functions are not in `tools/perfbench`.
- Windows behaviour is verified by unit tests on Linux and by the checklist in [windows-status](../design/windows-status.md) on Windows.

## Rejected alternatives

- Option A and Option C, above.
