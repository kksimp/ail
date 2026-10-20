# 0271. Every fuzz crash becomes a regression case; large ones are listed with their seed

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy (tool)
- **Spec rules:** `type.str.15` (found by the first probe run)
- **Related:** [ADR 0269](0269-fuzz-generator-design.md), [ADR 0270](0270-fuzz-properties.md)

## Context

A fuzzer that reports a failure once and forgets it protects nothing.

## Decision

- **`fuzz.policy.1`** Each crash found is minimised (the harness writes
  `tools/fuzz/failures/KIND-SEED.ail`, git-ignored) and committed as a case in
  `bootstrap/tests/cases/` named `fuzz_*`, with the `.rules`, `.trap` or
  `.out` that states the *correct* behaviour, in the same change as its fix.
- **`fuzz.policy.2`** A fix over about 50 lines is not made by the fuzzer's
  author in another lane's code: the PR body lists the seed, the minimised
  program and the owning lane, and the case is committed only when the fix is.
- **`fuzz.policy.3`** A fuzzing-only failure reproduces from
  `AIL_FUZZ_SEED=<seed> AIL_FUZZ_N=1 tools/fuzz/run.sh`; a message must name the
  property ID from ADR 0270.
- A memory-exhaustion abort is a crash: `type.str.15` makes `repeat` and
  `pad_left`/`pad_right` trap above 2^30 bytes (the first finding, by probe).

### Found tonight

| Finding | Seed / program | Resolution |
|---|---|---|
| `spawn p.sum()` (a method call on a value) passed `check` and panicked the interpreter | `fuzz_spawn_method_on_value` | `check/conc.rs`: the receiver must name a type (`conc.task.1`) |
| `"a".repeat(huge)` and `pad_left(huge, c)` aborted the process on allocation | probes 151, 260, 471, 683, … | `type.str.15` trap above 2^30 bytes |

## Rejected alternatives

Keeping a corpus directory of raw crashers (no explanation of the expected
behaviour; cases state it). Fixing all findings regardless of size (a large
change in another lane's file risks the merge it is waiting for).
