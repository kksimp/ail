# 1059. Shipped commands run no Unix program; std gets a temporary directory

- **Status:** Accepted
- **Date:** 2026-11-11
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`), toolchain, Windows parity
- **Spec rules:** std.fs.35, std.fs.37

## Context

Commands a user runs shelled out to `mktemp`, `rm`, `cp`, `mv`, `mkdir`, `sh`, `find`, `ls` and `env`: `ail test`, `ail diff`, `ail ask --affected`, `ail verify --diff`, `ail infer`, `ail task`, `ail shrink`, package fetch and the executable cache. On Windows those exist only when Git's `usr\bin` is on `PATH`, which the default Git install does not do, so `ail test` on a package failed there (`docs/windows.md` said so). Windows must be at parity with Linux and macOS before 0.1.0. The std gap behind it: `std.fs` has no temporary directory call, and `remove` refuses a non-empty directory (std.fs.6, ADR 0157).

## Options considered

### Option A: two std calls, tree removal in the tools (chosen)
`fs.temp_dir() str` (std.fs.35) and `fs.make_temp_dir(prefix) str ! IoError` (std.fs.37). No recursive remove in std: ADR 0157 stands.
- **Pros:** the only new surface is what the tools cannot build themselves (an exclusive create in the right place); `remove` keeps its meaning.
- **Cons:** each program carries a 25-line `remove_scrap`.

### Option B: `fs.remove_all` in std
- **Pros:** one call, no copies.
- **Cons:** contradicts ADR 0157 and std.fs.6; a program that passes a wrong path deletes a tree. Rejected.

### Option C: one shared module under `tools/common`
- **Pros:** one copy of `remove_scrap`.
- **Cons:** `use a.b` needs a directory under the program's package root (mod.use.4); a tool is one file and the compiler tree is its own root, so a shared module cannot be imported by both. Rejected; the copies are identical and each guards its own prefix.

### Option D: ship a tiny `rm`/`mktemp` with `ail`
- **Pros:** no source change in the tools.
- **Cons:** a second program per host to build and test, for what `std.fs` can do. Rejected.

## Decision

1. **std.fs.35 `temp_dir`:** `TMPDIR`, else `TEMP`, else `/tmp`; empty counts as unset; never fails.
2. **std.fs.37 `make_temp_dir(prefix)`:** a new empty directory in `temp_dir()`, named `prefix` and 12 random hex digits, mode `0700`, created with an exclusive `mkdir` (a taken name is retried with new digits, up to 64 times, then `exists`). The digits come from the system generator, never the seeded `std.random`.
3. **`remove_scrap`:** every program that removes a tree does it through one helper that only removes a directory whose last name part is the program's own scratch prefix (`ail-infer-`, `ail-verify-`, `ail-rev-`, `.ail-test-`, `.fetch-`). It walks with `fs.walk` and `fs.list_dir`, treats what `fs.walk` lists (files and links) as removable entries, so a symbolic link to a directory is removed as a link and never followed.
4. **Windows read-only files:** `fs.remove` of a file with the read-only attribute clears it and retries once, as `unlink` on Unix ignores the file's own mode. Git's object files are read-only; without this `remove_scrap` could not clear a clone.
5. **Compiler sources (`compiler/api`, `compiler/pkg`, `compiler/main.ail`) do not call the new std functions:** they are built by the published seed, which does not know them. They make their scratch directory with `fs.make_dir` and a name from the clock and `random.int`; the tools (built by the tree's compiler) call `fs.make_temp_dir`. A later seed lets the compiler switch.
6. **`ail test` scratch copy:** it must sit beside the package so relative `dep ... path ../x` lines resolve, so it is `DIR/../.ail-test-CLOCK-CHANCE` made with `fs.make_dir` and filled with `fs.copy`, not in `temp_dir()`.
7. **Replacements:** `ls -t` is `fs.metadata` modification times; the `sh -c 'find ... -newer'` staleness test is `fs.walk` and `fs.metadata`; `mkdir -p` is `fs.make_dir`; `mv` is `fs.rename`; `cp -R` is `fs.copy`; `rm` is `fs.remove`; `sh -c "git archive | tar -x"` is `git archive -o FILE` then `tar -x -f FILE` (Windows' own `tar.exe`, as `ail diff` already did).
8. **Kept, on Unix only:** `uname` (host target; Windows reads `PROCESSOR_ARCHITECTURE`) and `env VAR=...` for a child's environment, until a child-environment parameter exists. `chmod 755` of a built executable and `a-w` on a fetched dependency use `fs.set_mode` and `fs.mode` (std.fs.28-29, ADR 0977) once the seed that has them is pinned.
9. **Windows replacements for `env`:** `ail test` sets a test process's variables with `cmd /d /c set K=V&& exe arg`; package fetch runs `git` directly with a 10-minute timeout instead of `GIT_TERMINAL_PROMPT=0` (a credential prompt is then cut off by the timeout). Both are unverified on a Windows host until the `cli` case below runs there.
10. `git` stays: it is a documented requirement.

## Consequences

`ail test` (package and single file), `ail diff`, `ail ask --affected`, `ail verify --diff`, `ail infer --contracts`, `ail task`, `ail shrink` and package fetch need only `git` and `tar` on Windows. `docs/windows.md` and the README lose the `mktemp` requirement. The gate (`tools/gate/`) is not a shipped command and still uses `sh`. `fs.make_temp_dir` and `fs.temp_dir` are recorded and replayed (ADR 0873).
