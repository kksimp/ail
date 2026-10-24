# 0881. `ail ask`: semantic questions answered from the checker's graph

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076), within the item the designer approved in [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) (item 1)
- **Topic:** 22 — Tooling
- **Spec rules:** api.ask.1-6

## Context

An agent that wants to know who writes a field, what calls a function or which effects
reach `main` reads files or greps. That costs tokens and misses what the checker already
knows: resolved names, receivers that were mutated, inferred effects. `ail api --refs`
answers the question for one position; there is no question by meaning.

## Options considered

### Option A: `ail ask PROGRAM QUESTION...` over the checker's own facts (chosen)
- **Pros:** one command, plain words, answers from resolved symbols (no false hits on a
  same-named field of another type); no new analysis, no cost when unused; token-lean
  output (one line per site).
- **Cons:** a small fixed set of questions; it needs the program to check without errors.

### Option B: a query language (`ail query 'writes(Cart.total)'`)
- **Pros:** composable, unambiguous.
- **Cons:** a grammar to learn and document for every agent, so more tokens per question
  than the plain words the agent would write anyway.

### Option C: more `ail api` flags (`--writers FIELD`)
- **Pros:** no new command.
- **Cons:** `api` is for declarations; each question needs its own flag, which is the
  query-language cost again, spread across flags.

## Decision

`ail ask PROGRAM QUESTION... [--json]` answers `who writes F`, `who reads F`, `what calls f`,
`what does f call`, `which effects reach f` and `who uses T` from the checker's graph
(api.ask.1-6). The checker records, in `require_mutable` (the one place every assignment,
`mut` argument and mutating receiver passes), the field names a write reaches. The
references of `ail api --symbols` carry their enclosing function. Native only: it costs
nothing in a build or run that doesn't use it.

## Rationale

Token usage ranks first (ADR 0076): a one-line answer replaces reading several files.
Plain words cost the fewest tokens to write and need no documentation beyond the six verbs.
Reusing `require_mutable` keeps writes exactly as the ownership rules see them, so the
answer cannot disagree with the checker.

## Consequences

Edit impact and `ail test --affected` (ADR 0868 item 3) reuse the same graph. New verbs
extend `ask_verb`. A question the verbs can't express is an error that lists the six forms.

## Rejected alternatives

- **A query language:** more to learn and more tokens per question than plain words.
- **Flags on `ail api`:** one flag per question, the same cost spread across flags.
