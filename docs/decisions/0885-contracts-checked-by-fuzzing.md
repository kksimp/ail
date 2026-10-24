# 0885. Contracts checked by fuzzing: `ail test --contracts`

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076), within the item the designer approved in [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) (item 5)
- **Topic:** 22 — Tooling
- **Spec rules:** contract.fuzz.1-3

## Context

A failing `requires` or `ensures` traps with a message and a position
([contract.ens.1](../spec/20-contracts.md)). The agent that wrote the function still has
to find an input that makes it fail. Property tests (ADR 0198-0199) already generate and
shrink inputs, so a failing contract can come with the smallest input that breaks it.
[OQ-060](../open-questions.md) keeps static proof open.

## Options considered

### Option A: a property test per contracted function (chosen)
- **Pros:** reuses the generator, the seeds and the shrinker, so the output is the same
  `input: n = 1` an agent already reads; no prover; costs nothing in a build or run that
  doesn't ask for it.
- **Cons:** finds counterexamples, doesn't prove there are none; only parameter types the
  generator knows.

### Option B: a static prover (an SMT solver or a verifier)
- **Pros:** a proof, not a sample.
- **Cons:** a large dependency or a prover to write, against ADR 0009's cost; OQ-060 (b)
  recommends waiting.

### Option C: run `ensures` only on the calls a program makes
- **Pros:** nothing new.
- **Cons:** the status quo: a trace, and only the inputs the program happens to use.

## Decision

`ail test --contracts FILE` makes a property test for each free function with `requires`
or `ensures` lines: an input that fails a `requires` is discarded, the others call the
function, and a failing `ensures` (or a callee's contract) is reported with the shrunk
input (contract.fuzz.1). Methods, generic functions, `mut`/`move` parameters and
ungeneratable types are not checked (contract.fuzz.2). A function with effects is skipped
so checking does no I/O (contract.fuzz.3).

## Rationale

Token usage ranks first (ADR 0076): a counterexample is one line where a trace is many,
and the agent can paste it into a test. Reusing the property machinery adds no new
generator to keep in step with the language. Skipping effectful functions keeps the tool
safe to run on any file.

## Consequences

Static checking stays open under OQ-060 (b). Invariants and methods can follow by
generating the receiver the same way. A function whose only fault is a caller breaking its
`requires` is found by fuzzing the caller, which needs a contract or an explicit test.

## Rejected alternatives

- **A static prover:** a large dependency, and OQ-060 recommends waiting.
- **Checking only the calls a program makes:** the status quo, with a trace and no search.
