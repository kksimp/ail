# 0925. Windows: a Unix-shaped gate, and native, examples and run green before each release

- **Status:** Accepted (amends [ADR 0842](0842-mac-first-gate.md); refines [ADR 0004](0004-macos-arm64-only.md) for Windows testing)
- **Date:** 2026-10-27
- **Decided by:** Designer (2026-10-27, "2B")
- **Topic:** 24 — Cross-platform ABI strategy
- **Spec rules:** none

## Context

ADR 0004 makes Windows a first-class target, and ADR 0842 calls Windows testing
best-effort. On ail-win the selftest had about 700 REDs. Triage showed most were
the gate harness itself (it is written for a POSIX shell, `./ail` and `/` paths),
not product bugs. The rest were real Windows faults, now mostly fixed.

## Options considered

### Option A — port the whole gate harness to Windows
- **Pros:** one gate for all three systems.
- **Cons:** large work in harness code that ships to no user; every new suite pays it again.

### Option B — keep the harness Unix-only, and require the product suites on Windows before a release (chosen)
- **Pros:** tests what users run (native builds, the examples, `ail run`) without porting test plumbing; per-change gating stays fast.
- **Cons:** a Windows-only fault can reach `main` and is caught at release time at the latest.

### Option C — leave Windows best-effort
- **Pros:** no added work.
- **Cons:** contradicts ADR 0004's first-class Windows target.

## Decision

The gate harness stays Unix-shaped: `tools/gate/mac-gate.sh` on the Mac gates
every change, and Linux runs on ail-linux. Before every release (a published
seed or a tagged version), the `native`, `examples` and `run` suites must pass
on Windows (ail-win), built with that tree's own stage-2 compiler.

## Rationale

It meets ADR 0004 where users meet Windows, the compiler and the programs it
builds, while keeping test plumbing in one shape.

## Consequences

- A release is blocked while any of the three suites is RED on ail-win.
- Harness-only failures on Windows (shell, `./ail`, path handling) are not
  release blockers and are fixed only when cheap.
- Windows runs per change stay optional; lanes touching Windows runtime arms
  still run them.

## Rejected alternatives

- Port the harness (option A): cost in code no user runs.
- Best-effort only (option C): leaves the first-class target untested.
