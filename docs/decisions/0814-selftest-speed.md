# 0814. Selftest speed: parallel native builds, then parallel test binaries, then a shared compiler load

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), approved by the integrator
- **Topic:** Tooling: the integration gate (ADR 0085)
- **Spec rules:** none; test harness only

## Context

`./ail selftest` takes about 13 to 15 minutes on Linux x86-64 and runs at
every integration gate. A timing study on the batch head (day 3, serial run,
4 cores) measured 767 s for the 46 test binaries, after a 147 s cold build.
The native test binary alone took 413 s, and 380 s of that was one test,
`executables_match_the_interpreter`. It builds 140 programs one after another.
Each cold `ail build` costs about 3.5 s, and about 0.7 s of that is the
interpreted native backend re-checking all of `compiler/` for every program.
Cargo runs the test binaries one at a time, so three of four cores sit idle
through the native binary.

## Decision

Three steps, each its own pull request, in this order:

1. **Native builds on four threads.** `executables_match_the_interpreter`
   checks its programs on four workers. Every program writes only to paths
   named after itself (its executable and its own `os_*` scratch directory),
   so programs cannot race. Failures keep their index and are reported in
   sorted order, so the message matches a serial run. The set of programs is
   unchanged.
2. **Test binaries as parallel workers.** `./ail selftest` runs the test
   binaries on three or four workers, native first. Every test that uses a
   port, a temporary directory or `AIL_BINARY` must choose a unique one for
   each run. The walker-tier test keeps its 60 s budget. The PR lists every
   test checked for ports, temporary directories and `AIL_BINARY`, and what
   changed. The gate runs three times on the final commit to shake out flakes.
3. **One compiler load for many builds.** `ail run compiler build-many RT
   TARGET MS MB FILE...` builds several programs in one run, so `compiler/`
   loads and checks once per batch instead of once per program. Each
   executable is printed as `== PATH` and its hex, as `build` prints one. The
   target code is one function shared with `build`, and the runtime is read
   once, so every program gets the same run limits (`run.limit.5`) baked in.
   It is an internal mode of `ail run compiler`; `ail build`, `ail run` and
   the build cache are unchanged. Step 1's four workers each build their own
   quarter of the programs in one batch, so the compiler loads four times
   instead of 140. Each file is checked with `ail check` first, as `ail build`
   does. If a batch fails, its programs are built one at a time, so the failure
   is still reported per program. Native rules: `native.build.3`, `run.limit.5`
   (output unchanged). A test checks five programs byte for byte against
   separate `ail build` runs.

Step 2 (`./ail selftest` with no arguments in `ail`): cargo builds the test
binaries, then the native test binary runs alone, since it already uses four
threads, and the other 45 binaries run on four workers. After the first failure
no new binary starts; the logs print in cargo's order. Tests that take a
temporary path use the binary's own name or the process id, so runs cannot
collide. Tests that pass a port use port 0. `AIL_BINARY` is read-only. A run
with extra arguments still goes straight to `cargo test`.

Measurements and the full table are in the day 3 selftest timing report
(`reports/selftest-timing-day3.md` in the project files).

## Rejected alternatives

- **Parallelise inside each program's build instead:** the cost is per
  program and sequential, so splitting one build would not cut the total.
- **Shrink the set of programs checked:** would lose coverage, and the gate
  must check the same set as before.
- **Make the build cache survive `compiler/` edits:** the key covers the whole
  directory on purpose (a change to the backend changes the bytes), and
  during batch churn every merge changes `compiler/`, so a finer key gives
  little in the gate.
- **One batch for all 140 programs:** not measured. Four batches of 35 match
  step 1's four workers, and a failure in one batch only moves its quarter to
  per-program builds. Four batches measured 19.7 s for the native test against
  73.4 s for step 1 alone.
- **A separate lane for step 3:** the change is an internal mode the selftest
  uses, not a change to `ail run` for users, so it is decided here.
- **Run all test binaries in parallel at once:** the box has four cores, and
  unbounded parallelism would starve the native binary and make timings
  unreliable. Three or four workers bounds it.
