# 0505. Windows process lookup tries PATHEXT in std.proc.run

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard modules (Windows)
- **Spec rules:** [std.proc.1](../spec/17-std.md), [std.proc.3](../spec/17-std.md)
- **Criterion that decided:** Understanding (criterion 2) and token use (criterion 1). A program that works on Linux (`npm`, `git`) should work on Windows with the same source. Otherwise agents write per-platform code.

## Context

Rust's `Command::new` on Windows finds `.exe` files only. npm, pnpm, yarn and
many other tools install as `.cmd` batch files. `std.proc.3` (`which`) already
tries `PATHEXT`, but `std.proc.1` (`run`) did not, so `proc.which("npm")` found a
program that `proc.run("npm")` then could not start.

## Options considered

### Option A — leave `run` as it is
- **Pros:** no change.
- **Cons:** `which` and `run` disagree on Windows, which the spec says they must not.

### Option B — resolve bare names with `which` before spawning, on Windows only (chosen)
- **Pros:** one function, and `which` and `run` now agree. Linux is untouched.
- **Cons:** a `.cmd` file runs through `cmd.exe`, so an argument with characters such as `%` or `"` fails. Rust refuses these (the fix for CVE-2024-24576), and the failure is a `ProcError`, which is the honest result.

### Option C — always run `cmd /c` on Windows
- **Pros:** every name works, including shell builtins.
- **Cons:** an extra shell changes quoting and exit codes, and the spec says "without a shell" (std.proc.1).

## Decision

Option B. On Windows, a bare name without a separator is resolved with the `which`
search, and a name that isn't found is passed on as it is, so it fails as
`not_found`, as on Linux.

## Consequences

- Not testable on Linux. The Windows checklist in [windows-status](../design/windows-status.md) runs `npm --version` through `std.proc`.
- The `std_proc` case uses `echo` and `sh`, which Windows lacks. The case is POSIX-only and the checklist says so.

## Rejected alternatives

- Option A and Option C, above.
