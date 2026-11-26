# 1189. The edit oracle: `ail test` runs old against new on generated inputs

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** Designer (build idea 3 of the edge list, 2026-11-25); the details by AI (delegated, [ADR 0076](0076-delegated-decisions.md)); no syntax change
- **Topic:** 28 — Testing strategy
- **Spec rules:** [test.oracle.1-6](../spec/19-testing.md), [api.impact.7](../spec/22-tools.md)
- **Builds on:** [ADR 0882](0882-edit-impact.md) (edit impact), [ADR 0198](0198-test-properties.md) and [ADR 0199](0199-property-seeds-and-shrinking.md) (generated inputs, shrinking)

## Context

Every edit raises one question: what else did it change? Tests answer it only for the behaviour somebody wrote a
test for. The review of the agent trials found AIL's advantage is on changes to existing code, and that reports an
agent "must check rather than trust" do not help it. The parts of a direct answer already existed: effect inference
says which functions are pure and repeatable, property tests generate and shrink inputs from parameter types, the
change graph (`ail ask --affected`) says which functions an edit reaches, and a build takes a fraction of a second.
`ail verify` and `ail test --affected` exist and agents do not find them, so the answer has to arrive in a command
they already run.

## Decision

- **No new command.** A plain `ail test FILE` runs the oracle when `FILE` differs from git's `HEAD` copy
  (test.oracle.1). `--against OLD` names the older version by path, for a directory with no repository
  (`ail ask --affected --against OLD`, api.impact.7).
- **What is compared** (test.oracle.2): free functions of `FILE` that the edit changed or that reach a changed
  declaration, with the same header in both versions, a result, no effects (a `!` failure is allowed) and generated
  parameter types. A function whose signature changed is therefore compared through its unchanged callers. At
  most 8 functions, nearest the change first.
- **How.** Each version is built once: the file without its tests and `main`, plus a `main` that calls function K
  on every input and writes the result's text. The old version is built in git's copy of the directory, unpacked
  beside it. One process runs all the inputs of a function; a trap ends it, is recorded as that input's outcome, and
  the rest run in the next process.
- **Inputs** (test.oracle.3) are a property test's, by the same seed rule, with numbers past 100000 brought near
  zero so a counted loop does not use the time allowance. The first differing input is shrunk, 40 runs at most.
- **Noise** (test.oracle.4): a function that a test changed or added by the same edit calls is left out, because
  the edit states its new behaviour. A caller is left out when its path to the change goes through a function
  already reported. A function that passes the time allowance is left out.
- **Output** (test.oracle.5): information lines after the summary, never a failure:
  `changed behaviour: fee(amount: 1, rate: 50) was 0, now 1 (61 of 100 inputs differ)` and one
  `same behaviour: NAMES` line. JSON lines carry `"oracle"`.
- **Off switch** (test.oracle.6): `--no-oracle`, `AIL_TEST_ORACLE=0`, and any run that selects, records or
  rewrites. `AIL_TEST_ORACLE=why` explains a silent run.

## Options considered

- **A new command (`ail oracle`, `ail diff --behaviour`):** rejected. Agents in the trials ran `check`, `build`,
  `run` and `test` and nothing else.
- **Both versions in one executable, the old functions renamed:** rejected for now. Everything the old function
  reaches (types, constants, callees, modules) would need renaming too; two builds are simple and exact.
- **One process per input, as property tests run:** rejected. 100 inputs for 8 functions in two versions is 1600
  processes. A batch per function is two.
- **Treat a difference as a test failure:** rejected. Most edits intend a change; a failing status would teach
  agents to turn the oracle off.
- **Run it inside `ail check`:** rejected; `check` stays in the tens of milliseconds. Affected tests inside `check`
  is a separate decision.
- **Compare functions with effects by record and replay:** deferred. The recorder exists (ADR 0398), but a replayed
  old recording does not say what the new version would have read.
- **Compare against the last version `ail test` saw, when there is no repository:** deferred; `--against` covers a
  directory without git until that is measured.

## Consequences

- `tools/ailtest/ailtest.ail` (one section, `--no-oracle`, `--against`), `compiler/api/ask.ail` (`--against`),
  `tools/gate/gate.ail` (`Gate.edit_oracle`, in the `diff` suite), the fixture `tests/cases/packages/test_oracle`.
- A plain `ail test` on an edited file takes two more builds and a few runs, about the time of the test run
  itself. An unchanged file pays one `git diff`.
- Limits: methods, generic functions, functions with `mut` parameters or effects, and edits that are only in
  an imported module are not compared yet. Numbers past 100000 are not tried.
- Unmeasured: how often agents break untested behaviour in change tasks. The frozen change tasks of the final
  run can answer it with a free replay.
