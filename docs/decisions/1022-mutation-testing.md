# 1022. Mutation testing: `ail test --mutate`

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076). The slice is row 1022 of the AI-first plan of 2026-11-09.
- **Topic:** 22 — Tooling
- **Spec rules:** test.mutate.1-5
- **Supersedes:** [0896](0896-ail-test-mutate.md)

## Context

A passing test suite says nothing about how much it checks. Line coverage ([ADR 0871](0871-native-instrumentation-layer.md),
`test.cover.1-4`) says a statement ran, not that a test would notice if it did the wrong thing.
An AI agent writes code and its tests in one pass and tends to write tests that run the code and
assert little. Mutation testing finds that: change the program in one small way, run the tests,
and report the change if every test still passes (a surviving mutant). cargo-mutants and
PIT do this for Rust and Java.

## Options considered

### Option A: source-line mutants, judged by `ail test` on a copy (chosen)
- **Pros:** no compiler change; the mutant is an ordinary program, so the checker rejects a
  mutant that no longer type-checks (an invalid mutant) and every existing `ail test` feature
  applies; the report names `file:line` and the exact change, which an agent can act on.
- **Cons:** one build and run per mutant, so the cost is bounded by `--max`, not by the program;
  operator swaps are textual, so a few are equivalent mutants (`<` to `<=` on a value that is never equal).

### Option B: mutate MIR inside the compiler and run all mutants in one binary
- **Pros:** fastest; no rebuild per mutant.
- **Cons:** a new compiler pass and a switch table in every build; the report would name MIR, not source. Worth revisiting if the build per mutant proves the limit.

### Option C: random mutants
- **Pros:** trivial.
- **Cons:** not reproducible, which an agent re-running to see a fix needs.

## Decision

`ail test --mutate [--max N] FILE` runs the tests once. If they pass, it takes the lines the
tests reached (the native `lines` instrumentation), makes mutants of them, runs the tests on each
mutant and reports the survivors as `survived FILE:LINE: CHANGE (in FN)`. Mutants are in source
order and, past `--max` (default 50), spread evenly, so a run is deterministic. The exit status
is 1 when a mutant survived (test.mutate.1-5).

## Rationale

Token usage (ADR 0076 criterion 1): the report is one line per weak spot, with the line and the
change, so the agent adds an assert for it with no further reading. Reusing `ail test` on a copy
keeps the new code to the mutant generator and the loop.

## Consequences

- Only the file given is mutated; tests in the module directories it uses are not run on the
  mutants, and a mutant in another file is not made.
- A mutant that loops forever is killed by the per-test time limit (5000 ms, or `--timeout`).
- An equivalent mutant is reported as surviving. The score is killed over killed plus survived.
- Not yet: mutating callees in other files, `--function NAME`, parallel mutant runs, and skipping a
  mutant by the test that covers its line.

## Rejected alternatives

- **MIR mutation (B):** a compiler pass for speed; deferred until the build per mutant shows as the limit.
- **Random mutants (C):** not reproducible.
- **Mutating the tests too:** a failing test would be a "killed" mutant that says nothing.
