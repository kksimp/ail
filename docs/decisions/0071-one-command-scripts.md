# 0071. Everything a person runs is a one-command, self-setting-up script for every OS

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process; 27 — Build system (developer experience)
- **Spec rules:** none

## Context

Running the token benchmark by hand needed seven OS-specific commands. The
designer pasted the Windows variant into macOS, and Homebrew Python refused
system-wide installs. The designer asked for scripts that do everything,
because automated tooling is a project goal anyway.

## Options considered

### Option A — One self-setting-up script per OS family (chosen)
- **Pros:** One command to run. No environment knowledge needed. Can't pollute
  the system Python. Works the same for designers, contributors, CI and AI agents.
- **Cons:** Two scripts (bash and PowerShell) to keep in sync per tool.

### Option B — Documented manual steps
- **Pros:** Nothing to maintain.
- **Cons:** Error-prone, and OS-specific mistakes, as happened here.

## Decision

Every tool or procedure that the designer, a contributor, or CI needs to run
ships as:
- `run.sh` for macOS and Linux (bash), and
- `run.ps1` for Windows (PowerShell, which also works under PowerShell 7 on macOS and Linux).

Each script:
1. Sets up its own private environment on first run (never system-wide).
2. Skips setup on later runs, and redoes it automatically when dependencies change.
3. Accepts paths relative to the caller's directory.
4. Fails with a clear, actionable message (missing interpreter, missing
   credentials, and so on).
5. Never uses paid services or credentials unless explicitly asked by a flag.

## Consequences

- AI agents never ask the designer to type multi-step setup commands. They
  provide or extend a script instead (rule added to AGENTS.md).
- `tools/tokbench/run.sh` and `run.ps1` are the first instances.
- Once AIL's own toolchain exists, `ail` subcommands take over this role
  (e.g. `ail bench tokens`).

## Rejected alternatives

- **Manual steps:** proven error-prone on the first attempt.
