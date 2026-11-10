# 1020. `ail context`: a token-budgeted bundle for an edit

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076), implementing the item the designer chose in [ADR 0891](0891-ail-context-budgeted-pack.md)
- **Topic:** 22 — Tooling
- **Spec rules:** api.ctx.1-7
- **Builds on:** [ADR 0881](0881-ail-ask.md), [ADR 0882](0882-edit-impact.md), [ADR 0394](0394-token-budgets-on-every-command.md)
- **Amends:** [ADR 0891](0891-ail-context-budgeted-pack.md) (the section order; the command is built)

## Context

Before an agent edits `f` it reads the file, the callers, the callees' signatures,
the types and the tests, one file at a time. Most of those tokens are not needed, and
each read is a round. The checker already holds all of it (`ail ask`, edit impact).

## Options considered

### Option A: `ail context PROGRAM SYMBOL --budget N` over the checker's graph (chosen)
- **Pros:** one call; every part comes from resolved names, not text search; the budget
  is the agent's own choice between completeness and tokens; no new analysis, so no cost
  when unused.
- **Cons:** the program must check without errors; only functions and methods are symbols.

### Option B: a per-file outline (`ail api`) plus the agent's own reads
- **Pros:** exists today.
- **Cons:** the callers, tests and rules are still separate reads, which is the cost to remove.

### Option C: the whole call graph to depth N
- **Pros:** simple to state.
- **Cons:** grows with the program, not with the edit; most of it is never needed.

## Decision

`ail context PROGRAM SYMBOL [--budget N] [--json]` prints one pack (api.ctx.1-7). Its
sections, in the order they are kept: the signature with contracts and doc, the body,
the effects with their chains, the callees' signatures, the callers' signatures, the
types it names, the tests that reach it and the spec rule IDs its comments cite. The
longest prefix of the items that fits the budget is shown, never an item cut in the
middle; the last line says what was cut and the budget that shows all. The default
budget is 2000 (ADR 0891). The signature is shown even when it alone is over the budget.

**Order, amending ADR 0891.** ADR 0891 dropped the callees first. The lane plan of
2026-11-09 puts the callees' and callers' signatures before tests and rules: when
editing a function, the signatures decide whether a call is right, and the tests and
rule IDs only point to more to read. Types and effects are added to the pack because
the edit's lane plan asks for the types it uses, and an effect is one line.

**Counting.** The estimate of `ail api --card` (api.card.3), so a pack costs what `--budget` elsewhere counts.

**Not in the first version.** The text of a cited rule (only its ID is given; `ail explain RULE` has the text) and the
callers of a trait method through the trait.

## Measurement

The slice's tokbench task is "add a field to a struct and update the functions that use it". The before-and-after
token counts are added here once a Mac run has measured them.

## Rationale

Token usage ranks first (ADR 0076): one call replaces five reads, and the agent
chooses its own cost. A prefix is predictable. An agent that sees `cut: tests 2` knows
what it is missing and what to ask for.

## Consequences

The command is native only, like `ail ask`. `ail mcp` can serve it later as `ail_context`. New sections
extend `context_pieces` and the order in api.ctx.2.

## Rejected alternatives

- **Cutting the longest section:** a pack's shape would change with the program, not the budget.
- **Cutting inside an item:** a half-body misleads more than a missing one.
- **A raw window of lines around the function:** includes unrelated code, and its cost grows with the file.
