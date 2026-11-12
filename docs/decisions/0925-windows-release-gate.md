# 0925. Windows: the full gate green before each release

- **Status:** Accepted (amends [ADR 0842](0842-mac-first-gate.md); refines [ADR 0004](0004-macos-arm64-only.md) for Windows testing)
- **Date:** 2026-10-27, amended 2026-11-11
- **Decided by:** Designer (2026-10-27, "2B"; amended by the designer's parity rule of 2026-11-10)
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

As first decided (2026-10-27, option B): the harness stays Unix-shaped, the Mac
gate gates every change, and the `native`, `examples` and `run` suites must pass
on ail-win before a release.

**Amendment (2026-11-11).** The designer ruled on 2026-11-10 that Windows, Linux
and macOS must be at feature parity and that the Windows gate must be green for
a release. So, before every release (a published seed or a tagged version):

- `tools\gate\win-gate.ps1` must end `GATE GREEN` on ail-win, on a fresh LF
  clone, built with that tree's own stage-2 compiler. It runs the suites of
  `tools/gate/suites.txt`, the one list `tools/gate/mac-gate.sh` also runs on
  the Mac and on Linux.
- `tools\self-host\run.ps1` must reach the fixed point there (stage 2 builds
  stage 3, the same bytes).
- The `parity` suite (`tools/gate/parity.ail`) prints one line per feature; the
  lines must be the same on the three hosts but for those listed in
  `tests/parity/expected-differences.txt`, which holds only what the operating
  system itself decides and what waits for the next seed.

The harness still runs through Git for Windows' `sh` and `perl`; it was fixed
where it failed rather than ported (option A's cost did not have to be paid in
full). The Mac gate still gates every change; Windows runs per change stay a
subset.

## Rationale

It meets ADR 0004 where users meet Windows, and since the amendment one suite
list means a feature cannot be tested on two hosts and forgotten on the third.

## Consequences

- A release is blocked while any suite is RED on ail-win, harness failures
  included: they are fixed, not waived.
- A new suite goes into `tools/gate/suites.txt` and so runs on the three hosts.
- A feature one host cannot have is a line in
  `tests/parity/expected-differences.txt` with its reason, and nothing else is.

## Rejected alternatives

- Port the harness to PowerShell (option A): cost in code no user runs; the amendment runs the Unix-shaped harness on Windows instead.
- Best-effort only (option C): leaves the first-class target untested.
- Three suites only on Windows (the first form of this ADR): Windows-only faults in the other suites reached release candidates, and the designer's parity rule replaced it.
