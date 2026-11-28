# 1307. An outside-test kit, and `"n" + "{x}"` joins in one certain fix

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for an outside run with ChatGPT in place of a paid confirmation run
- **Topic:** 22 — Tools
- **Spec rules:** [trait.op.1](../spec/10-traits.md), [fix.auto.1](../spec/22-tools.md)

## Context

The designer chose an outside agent (ChatGPT, run by hand on his machine) over a paid confirmation run. It needs the
agentbench build and maintenance tasks, in AIL and C, with a scorer, packed so it runs from one command. Separately,
`"n" + "{x}"` got no certain fix: the string-join fix refused any literal with a `{`.

## Decision

- `tools/outside-test/` holds the kit: `run.sh` sets it up, `outside.py` lays out the tasks for the outside agent and
  scores its answers the way agentbench does ([ADR 0916](0916-agent-end-to-end-benchmark.md)), so the numbers compare
  with the earlier runs.
- The `+` join fix of `trait.op.1` stays certain when a literal's every `{` opens a hole closed in the same literal,
  one level deep: `"n" + "{x}"` becomes `"n{x}"`, which has one meaning. A literal with an unclosed or nested `{`
  is still refused. The mistakes pair `string_plus_interpolated_literal` checks it.

## Rejected alternatives

- **A paid confirmation run.** Declined by the designer for now.
- **Make every join with a braced literal certain.** A `{` that does not close in the literal would change meaning
  once joined.
