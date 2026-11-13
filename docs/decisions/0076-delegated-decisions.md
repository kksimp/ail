# 0076. The AI decides routine design questions by a fixed priority order; serious ones go to the designer

- **Status:** Accepted; priority order amended by [ADR 1077](1077-decision-order-ai-understanding-first.md)
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process
- **Spec rules:** none
- **Amends:** the "Roles" rule in AGENTS.md (previously the AI asked about every question)

## Context

After ~75 decisions, the designer delegated routine decisions to the AI to
speed up Stage 1:

> "From now on I want you to make decisions in this order: Token Usage,
> AI understanding, Human readability, Speed. Bring it to me if it's
> something that is seriously going to affect things."

## Decision

### Priority order

When options conflict, the AI chooses by these criteria, in order:

1. **Token usage**, measured with `tools/tokbench` (ADR 0029, ADR 0070).
2. **AI understanding**: local reasoning, unambiguous meaning, precise
   diagnostics.
3. **Human readability.**
4. **Speed**: runtime performance first, then compile time.

A criterion decides only when the difference is **meaningful**. A token
difference under ~2% on representative code, or one confined to rare
constructs, is treated as a tie, and the next criterion decides. Every
delegated ADR states which criterion decided it and why the earlier ones tied.

### Escalation: what goes to the designer

The AI brings a question to the designer if any of these apply:

1. It would **change or contradict an accepted ADR** made by the designer.
2. It's **hard to reverse** once Stage 2 starts, e.g. a core semantic model
   (memory, concurrency, modules and packages, FFI, unsafe, compile-time
   execution power).
3. It **materially costs a lower-ranked criterion**. In particular, anything
   that would reduce achievable runtime performance in a way users can't
   opt out of (mandatory boxing, mandatory dynamic dispatch, unavoidable
   checks). C-class performance is a founding goal, so "speed last" is
   treated as a tiebreaker, not as permission to give performance away.
4. It involves **money, legal or licensing matters, accounts, publishing**,
   or other external commitments.
5. The criteria point in strongly different directions on a high-impact choice.

### Recording

Delegated decisions get full ADRs like any other, with
`Decided by: AI (delegated, ADR 0076)` and the priority reasoning. The
designer can override any of them at any time. The AI reports each batch of
delegated decisions in a short summary.

## Consequences

- AGENTS.md "Roles" and "How to raise design questions" are updated.
- Stage 1 moves faster. Designer attention goes to the decisions that matter most.

## Rejected alternatives

- **Keep asking every question:** too slow for the remaining ~20 topics.
