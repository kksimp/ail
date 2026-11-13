# ADR 1097: Faster gates: shards, one process per fence, a fast `run` on Windows

- Status: Accepted (amends [ADR 0842](0842-mac-first-gate.md) and [ADR 0925](0925-windows-release-gate.md))
- Decided by: AI (delegated, ADR 0076)
- Date: 2026-11-13

## Context

A batch needs one gate per host and is usually red a few times. Measured on the Mac (4 jobs): `cases` 444 s and
`spec` 422 s each held one job for the whole run, so the longest pole, not the work, set the wall time. Each fence of
`spec` and each case is one to three `./ail` processes; on Windows every `./ail run` also starts PowerShell (about a
second), which is why the Windows spec suite takes about 43 minutes. A red suite printed only its first 8 lines, so
a second failure cost a second round.

## Decision

- **Shards.** A suite name may be `NAME@K/N`: shard K (from 0) of N. `cases` and `spec` have them
  (`gate.ail`, `spec.ail`); `suites.txt` lists four of each, slowest first. Every item belongs to exactly one shard
  (`i % N == K`), shard 0 also runs the CLI checks and the rule-coverage check. `NAME` alone still runs the whole suite.
- **One process per runnable spec fence.** A clean fence with a `main` runs with `ail run` only; the separate
  `ail check` runs only when that fails, to report why. The pass/fail verdict is the same.
- **Stage 1 in the pool (Mac, Linux).** `selfhost1` is a job of the pool next to the long suites instead of a serial
  step before them; nativex, which needs it, starts after the pool. The refresh of `.native/ail` is the first build
  of `arith.ail`, which is also the "binaries start" check.
- **A cover check.** Each shard prints `SHARD name K/N total T ran R`; a full run is RED unless all N shards of a suite
  reported, agree on T, ran more than 0, and their R add up to T.
- **Pool size** is the core count, at most `GATE_JOBS_MAX` (default 4), so a VM with more cores gets a wider pool.
- **`mac-gate.sh --failed`** reruns the suites the last run left red. A release gate is still a full run.
- **All failures at once.** A red suite prints up to `GATE_SHOW` (default 60) of its `RED` and error lines.
- **Fast `run` on Windows.** With `AIL_FAST_RUN=1`, `./ail run FILE [ARGS]` under Git's sh does ail.ps1's check,
  prepare and start itself, with no PowerShell. win-gate sets it for every suite except `examples`, `cli`,
  `build_cache` and `docs`, so ail.ps1's `run` stays gated; `GATE_FAST_RUN=0` turns it off. The gate stays a full gate:
  the same suites, the same checks.

## Amendment 2026-11-13: nativex in the pool

The first Linux run (4 vCPUs, 1645 s) showed `nativex` (613 s) running alone after the pool, with its own phases
mostly serial, and `api_card` (490 s, one process) starting after the shards. `nativex` is now sharded the same way
(`nativex@K/N`: every n-th program of `executables`, every n-th of the other tests; one process each), the four
shards are the last jobs of the pool and wait for `selfhost1`, and `api_card` starts first. The cover check counts
nativex items too.

## Rejected alternatives

- **More jobs by default.** The Mac was overloaded into a reboot by too many concurrent processes; the default
  stays one job per core, at most 4. The VMs have 3 vCPUs, so more cores for them is a recommendation to the designer.
- **Skipping suites by diff.** A docs-only change can still break a card or a golden; only `--failed` reruns
  are offered, and the designer or integrator decides when that is enough.
- **A fast `run` for everyone on Windows.** It would hide ail.ps1 from the gate; kept opt-in and gated as above.
