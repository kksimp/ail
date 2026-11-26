# 1194. Tool reports are cross-checked by a fuzz gate

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the edge ideas to be built (idea 8)
- **Topic:** 22 — Tools
- **Spec rules:** none new; checks api.ask.2-4, api.impact.1-4 and 6, test.affected.1, verify.run.1-5, fix.apply.1-2, fix.auto.1-3
- **Builds on:** [ADR 0868](0868-ai-first-tooling-after-rust-removal.md), [ADR 0882](0882-edit-impact.md), [ADR 0909](0909-ail-verify-diff.md), [ADR 1112](1112-check-one-pass-and-fuzz.md), [ADR 1130](1130-certain-fixes-applied-by-check.md)

## Context

An agent acts on what `ail ask`, `ail ask --affected`, `ail test --affected`, `ail verify` and the fixes say
without reading the code behind the answer. One wrong answer costs more than no tool: the outside trial's
verdict turned on `verify` naming a function untested that a test reached (`reaches` listed one changed
declaration per test). Each tool had hand-written gate cases, and no case compared one tool with another or
with a real run.

## Options considered

### Option A — a generator that knows the answer, plus real runs (chosen)
- Generate small programs from a model that holds the call graph, the readers and writers of each field, the
  `io` effect of each function, every value and the output. Commit, apply one or two random edits to the
  model, write the file again, and compare each tool's report with the model and with fresh runs.
- **Pros:** an exact expected answer for every question, so a wrong extra answer fails as a missing one does;
  the real run, not the model, decides which tests fail; failures replay from a seed.
- **Cons:** the programs use a narrow part of the language (functions, constants, one struct, tests).

### Option B — random edits to corpus programs, tools compared with each other only
- **Pros:** real code. **Cons:** most random edits do not compile; with no known answer two tools that share
  one graph agree while both are wrong. Kept as a later addition, not instead.

### Option C — more hand-written cases
- **Cons:** the `reaches` bug sat beside hand-written cases for the same command.

## Decision

Option A: `tools/fuzz/tool_fuzz.ail`, rules fuzz.tool.1-6 in its header, run by the gate suite `toolfuzz` in
its fast form (20 programs, seed 1, about four minutes) and by hand with any count and seed. A failure keeps the program and its
base. `ail perf` is not covered: its remarks are the optimizer's own decisions (tool.perf.2) and a run shows
none of them directly.

## Consequences

- A tool change that alters `ask`, the impact JSON, `verify --json`, `diff --json` or the rewritten spellings
  must keep the suite green or change it in the same PR.
- The suite starts about twenty tool processes per program; it stays short by count, not by sharing work.

## Rejected alternatives

Options B and C above. Checking `reaches` only against `verify` (the pair that disagreed): it would pass
whenever both read the same wrong list.
