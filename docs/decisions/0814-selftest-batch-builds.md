# 0814. Batch native builds for the selftest

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), on the integrator's approval of an internal mode the selftest uses
- **Topic:** Native backend and selftest
- **Spec rules:** `native.build.3` (the output of `ail build`, unchanged), `run.limit.5` (the baked limits, unchanged)

## Context

`ail run compiler build` loads and checks all of `compiler/` (about 84 files)
on every run, about 0.7 s. The selftest builds about 170 programs, each in its
own run, so about 120 s is spent reloading the same compiler. The timing study
is `/mnt/project-files/reports/selftest-timing-day3.md`.

## Decision

`ail run compiler build-many RT TARGET MS MB FILE...` builds several programs
in one run. It takes the same arguments as `build` (the runtime, target and
run-limit defaults) and prints each executable as `== PATH` and its hex, as
`build` prints one. The compiler loads and checks once; the runtime is read
once, so every program gets the same baked limits. The executable code is one
function shared by `build` and `build-many`, so the two cannot drift.

The selftest's `executables_match_the_interpreter` builds its programs with one
`build-many` call. Each file is first checked with `ail check`, as `ail build`
does. If the batch fails for any file, the test builds each file on its own, so
the failure is still reported per file. `ail build`, `ail run` and the cache
are unchanged.

## Consequences

- The interpreter's process state is shared across the builds in one run. The
  compiler keeps no state between programs, and `batch_builds_match_separate_builds`
  checks five programs byte for byte against one `ail build` each.
- Run limits (`run.limit.5`) apply to the compiler run as they do to any `ail
  run`; the batch does not add counters that accumulate across programs.
- `compiler/main.ail` changes. `lower.ail` does not.

## Rejected alternatives

- **Reset the interpreter between builds in Rust:** would need an audit of
  process-wide state (limits, parallel, net, ffi, thread-locals). The AIL program
  is an ordinary run, so the audit is not needed.
- **Make `ail build` take many files:** changes the user-facing CLI. The batch
  is an internal `ail run compiler` mode instead.
- **Cache the compiler's load across runs:** ADR 0393 forbids a cached runtime.
- **Parallel test binaries first:** a larger change, and the reload cost would
  remain in each thread. Left for a later ADR.
