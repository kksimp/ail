# 1077. Delegated decisions put AI understanding first and human understanding second

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** Designer
- **Topic:** Project process
- **Spec rules:** none
- **Amends:** [ADR 0076](0076-delegated-decisions.md) (its priority order); resolves OQ-142
- **Builds on:** [ADR 1065](1065-goals-c-parity-ai-first.md)

## Context

ADR 0076 ranks token usage first. ADR 1065 accepts more tokens when they buy
speed or a better AI-first feature. OQ-142 asked whether the order should change.
On 2026-11-12 the designer wrote: "Never compromise on AI understanding.
Human understanding comes 2nd."

## Decision

When options conflict, delegated decisions are made by these criteria, in order:

1. **AI understanding**: local reasoning, no ambiguity, good diagnostics.
2. **Human readability.**
3. **Speed**: runtime, then compile time. ADR 1065's target is speed equal to optimised C.
4. **Token usage**, measured with `tools/tokbench`.

The designer set positions 1 and 2. The order of 3 and 4 follows ADR 1065,
which accepts more tokens for speed. Everything else in ADR 0076 stands: the
2% tie rule for tokens, the escalation list, and measurements behind syntax
choices (ADR 0029).

## Rejected alternatives

- **Keep ADR 0076's order (OQ-142 option A).** It conflicts with the designer's words.
- **Speed first (OQ-142 option C).** It would rank speed above clarity for agents, which the designer ruled out.
