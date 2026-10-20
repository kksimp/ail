# 0462. Every pure-subset program is built and diffed against both interpreter tiers

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy
- **Spec rules:** none (a tool); harness rule ID `fuzz.mod.6`

## Context

The native backend ([lane 120](../design/finish-plan.md)) is written in AIL
and is the only thing that can beat C. A wrong native binary is worse than a
slow one, and the interpreter is the permanent oracle ([ADR 0032](0032-keep-rust-interpreter-as-oracle.md)).

## Options considered

### Option A — differential in the existing selftest and fuzzers (chosen)
- **Pros:** one mechanism; every case, example and fuzzed program already runs under both tiers, so native is a third column.
- **Cons:** selftest slows by the build time; mitigated by building only programs the backend accepts.

### Option B — a separate native test suite
- **Cons:** two lists to keep in step; programs the suite forgets are untested.

## Decision

When `ail build` exists, the module fuzzers (`fuzz.mod.6`) build each program and
require the same stdout and exit code as the walker. Lane 120 adds the same
comparison for `bootstrap/tests/cases` and `examples` in `cases.rs` (programs it
rejects as outside the pure subset are skipped, and the skip count is printed).
Trap text is compared on stderr once the backend prints rule ids.
The command shape `ail build FILE -o OUT` is the contract; `AIL_FUZZ_BUILD_ARGS`
is not needed because `fuzz.mod.6` uses no flags.
