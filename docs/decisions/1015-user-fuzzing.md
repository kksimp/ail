# 1015. Coverage-guided fuzzing of user code: `ail test --fuzz`

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076). The slice is gap 7 of the replace-C/C++/Rust list (Kaleb, 2026-11-07).
- **Topic:** 22 — Tooling
- **Spec rules:** test.fuzz.1-5

## Context

`tools/fuzz` fuzzes the compiler ([ADR 0269](0269-fuzz-generator-design.md)-0271), and
`ail test --contracts` ([ADR 0885](0885-contracts-checked-by-fuzzing.md)) draws random inputs
for functions with contracts. Neither steers by coverage, and neither keeps a corpus, so a bug
behind a narrow branch of user code is found only by luck. cargo-fuzz and libFuzzer do both.
An AI agent that writes a parser or a codec needs the same tool to find its bugs.

AIL already has the pieces: property tests generate, run and shrink typed inputs
(`test.prop.1-5`), and the native instrumentation layer ([ADR 0871](0871-native-instrumentation-layer.md))
records which statements a run reached.

## Options considered

### Option A: a fuzz mode of `ail test` on the property machinery (chosen)
- **Pros:** the test is an ordinary `@test fn NAME(params)`, so it also runs as a property test;
  the generator, the shrinker, the report and `--bundle-failure` are reused; no new language
  feature and no new runtime code.
- **Cons:** one process per input, so thousands of runs a second are out of reach (libFuzzer runs
  in process). Only the parameter types of `test.prop.2`.

### Option B: a `fuzz` attribute and a bytes-in entry point
- **Pros:** the libFuzzer shape; one `bytes` input fits parsers.
- **Cons:** a new attribute (an ADR 0002 rule and a checker change) for what `@test fn f(b bytes)`
  says already; every structured input needs a hand-written decoder, which the property tests
  generate from the types.

### Option C: in-process fuzzing with a driver loop in the runtime
- **Pros:** fastest; no process start per input.
- **Cons:** a failure traps the task, so the loop would need to survive traps, which the runtime
  does not do; a large new runtime piece. Worth revisiting if the process cost proves the limit.

## Decision

`ail test --fuzz NAME [--time MS] [--cases N] [--seed S] [--corpus DIR] FILE` builds the named
property test with `AIL_INSTRUMENT=lines` and runs it on mutated inputs, keeping an input that
reaches a new statement and mutating the kept ones next. The first failure is shrunk with the
existing shrinker. `--corpus DIR` stores kept inputs and crashes as small text files, replayed
first on the next run. The report is the property test's report (test.fuzz.1-5).

## Rationale

Token usage (ADR 0076 criterion 1): the agent writes the same `@test` it already writes and adds
one flag; the output is the `input: n = 101` line it already reads. Reusing the property
machinery adds no second generator to keep in step with the language. The corpus format is
the argument tokens the test binary already decodes.

## Consequences

- A property test is also a fuzz target, with no change to the file.
- Speed is bounded by one process per input. If that becomes the limit, Option C is the next step.
- `--bundle-failure` records a fuzz failure, and `ail replay --shrink` ([ADR 0920](0920-replay-shrink.md))
  shrinks the bundle's recorded calls.
- Coverage is statements, not edges or values; mutation is structural, not byte-level.
- Not yet: running several fuzz tests in one call, a parallel fuzz (workers sharing a corpus),
  dictionaries, and comparison-guided mutation.

## Rejected alternatives

- **A `fuzz` attribute (B):** a new language surface for what a property test with a `bytes`
  parameter says already.
- **In-process loop (C):** needs a trap-surviving runtime; deferred until the process cost shows.
- **Random inputs only (the status quo):** finds nothing behind a narrow branch.
