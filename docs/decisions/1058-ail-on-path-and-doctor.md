# 1058. `ail` works from PATH; `ail doctor`

- **Status:** Accepted
- **Date:** 2026-11-11
- **Decided by:** AI (delegated, ADR 0076). Reasoning: AI understanding (the agent kit of ADR 1046 runs a bare `ail`), then human readability. Slice G2 of `lanes/final-gaps-2026-11-10.md`.
- **Topic:** 22 — Tools
- **Spec rules:** [tool.path.1, tool.doctor.1, tool.doctor.2](../spec/22-tools.md)
- **Builds on:** [ADR 1046](1046-agent-kit-in-ail-new.md)

## Context

`ail new` writes `.mcp.json` with `"command": "ail"` and an `AGENTS.md` whose every step is a bare
`ail ...`. Nothing put `ail` on PATH on macOS or Linux, and a symlink did not work: the launcher took
its root from the link's directory, so `fetch-seed.sh` was not found and the message blamed the seed.

## Decision

1. The `ail` launcher resolves symlinks (a portable loop, no `readlink -f`; the Mac has bash 3.2)
   before it takes its root, so `ln -s /path/to/ail/ail ~/.local/bin/ail` works (tool.path.1).
2. `ail doctor [--json]` prints one line per check, `ok` or a fix to run: toolchain root, version and
   seed tag, the first `ail` on PATH, `git`, `curl`, a hash tool, a writable build cache, and the
   std.app and std.audio hosts. Exit 1 when a required check (root, git, cache) fails (tool.doctor.1).
3. `ail doctor --path` prints the one line to add to the shell profile (`export PATH=...`, or a
   PowerShell line on Windows) and changes nothing (tool.doctor.2).
4. README Install and the tutorial carry a three-line "Put `ail` on your PATH" block per host, and
   `ail new` prints a one-line hint on standard error when `ail` is not on PATH.

## Rejected alternatives

- **An installer script that edits the profile.** It changes the user's files unasked.
- **An absolute path in `.mcp.json`.** It breaks when the project is shared.
- **Copying `ail` to a PATH directory.** The copy cannot find the toolchain root.

## Consequences

`ail.ps1` keeps `$PSScriptRoot` as its root: Windows puts the directory on PATH (`docs/windows.md`),
and a symlink needs privileges there. The doctor's check for the Git `usr\bin` directory is not
written: ADR 1059 removes the need.
