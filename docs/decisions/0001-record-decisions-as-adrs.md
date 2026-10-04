# 0001. Record design decisions as numbered ADRs

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process / documentation
- **Spec rules:** none

## Context

AIL will make hundreds of design decisions over years. Without a durable record,
people forget the reasoning, rejected options get proposed again, and AI agents
working across sessions lack context. The record must also be cheap for an AI
to load selectively.

## Options considered

### Option A — One numbered file per decision (ADR) (chosen)
- **Pros:** A permanent "why" for each choice. A single decision can be loaded
  without the rest. Supersession is explicit. Clean git history and diffs.
- **Cons:** Many small files. Superseded records need cross-links.

### Option B — A single living `DECISIONS.md`
- **Pros:** Simple, and everything is in one place.
- **Cons:** It grows without bound, so it is expensive to load into AI context.
  Individual entries are hard to link to or supersede. It causes merge conflicts.

### Option C — RFC process (proposal, then discussion, then decision)
- **Pros:** The deepest rationale, and it suits outside contributors later.
- **Cons:** Heavy for one designer, and it slows decisions.

## Decision

Each design decision gets its own file `docs/decisions/NNNN-short-title.md`,
following [0000-template.md](0000-template.md) and listed in the
[index](README.md).

## Rationale

Selective loading supports the token-efficiency goal. Explicit pros and cons and
rejected alternatives preserve reasoning for both humans and AI.

## Consequences

- Every decision the designer makes must produce an ADR before the work counts
  as done (enforced through [AGENTS.md](../../AGENTS.md)).
- ADRs are never deleted. A replaced decision is marked `Superseded by NNNN`.
- An RFC-style proposal stage can be added later if outside contributors join.
  This ADR does not prevent it.

## Rejected alternatives

- **Single log:** poor AI context efficiency, and individual entries can't be
  referenced.
- **RFC process:** too much ceremony for the current team size.
