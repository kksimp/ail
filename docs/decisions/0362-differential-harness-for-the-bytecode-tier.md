# 0362. `./ail selftest` runs every program on both tiers and the fuzzer compares them

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076), within the designer's ADR 0359
- **Topic:** testing; interpreter performance
- **Spec rules:** none (test harness); adds property `fuzz.prop.5` to [ADR 0270](0270-fuzz-properties.md)

## Context

[ADR 0359](0359-bytecode-tier-in-the-bootstrap.md) requires every program in
the test corpus to print the same bytes under the bytecode tier and the tree
walker. Slice 1 ([ADR 0361](0361-bytecode-tier-design.md)) checked that by
hand. The expectation files alone don't do it: `.trap` files name only the
rule, and checks that only diagnose never run.

## Options considered

### Option A — run each program both ways and compare the bytes (chosen)
- **Pros:** compares full standard error (trap text, snapshot, report) and the
  exit code, not only what an expectation file pins; needs no new files.
- **Cons:** every run happens twice (about 4 s on 4 cores for about 1,000 runs).

### Option B — run the existing expectation tests a second time with `--no-bytecode`
- **Pros:** reuses `cases.rs`.
- **Cons:** a difference in trap text or snapshot that no file pins goes unseen.

## Decision

- `bootstrap/tests/bytecode.rs` runs `ail run` (with a case's `.args`) on
  every example, test case, perfbench program, tokbench corpus program and
  package, and `ail test --json` on cases with a `.test` file, once with
  `--no-bytecode` and once without, and fails on any difference in stdout,
  stderr or exit code. `tests/deterministic/` (clock and unseeded random) is
  left out.
- The fuzz harness adds `fuzz.prop.5`: each generated program that checks
  clean runs with `--deterministic --no-contracts` (walker) and with `--deny
  clock,random,proc,net --no-contracts` (tier), and the two must match.
  `--deterministic` and checked contracts keep a run on the walker for now,
  so the tier's run denies the same capabilities instead.

## Rationale

Correctness of the oracle comes before the run's cost (ADR 0076, ADR 0032).
A seeded mutation of the tier (one wrong int result) failed 14 corpus runs and
2 of the 200 smoke fuzz programs.

## Consequences

- A tier bug shows up in `./ail selftest` on both platforms (ADR 0085).
- When slice 4 lets the tier run under `--deterministic` and contracts,
  `fuzz.prop.5` can compare the same command with and without `--no-bytecode`.

## Rejected alternatives

- Option B: it would miss differences no expectation file pins.
