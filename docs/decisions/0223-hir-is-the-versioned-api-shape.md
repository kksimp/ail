# 0223. The HIR's printed shape is the versioned API; MIR stays internal

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 19 — AST/IR architecture; 20 — AI semantic API
- **Spec rules:** none (tool interface; `api.*` in [semantic-api.md](../design/semantic-api.md))
- **Design note:** [ir-architecture.md](../design/ir-architecture.md) §3–4

## Context

ADR 0024 versions the semantic API from its first release, and ADR 0145 and
ADR 0150 give `ail check --json` and `ail api` an `api` key. With four tiers
(ADR 0222), it must be clear which tier agents may build on. **AI
understanding decides**: an agent needs one stable answer, not a moving
internal structure.

## Options considered

### Option A — version the HIR's printed shape; MIR and machine IR unstable (chosen)
- **Pros:** agents build on the level they reason at; the optimiser stays
  free; MIR facts reach agents as reports (allocations, decisions), which
  are additive keys.
- **Cons:** some agents will want MIR; they get an explicitly unstable dump.

### Option B — version MIR too
- **Pros:** tools can analyse optimised code.
- **Cons:** freezes the optimiser's internals for years.

### Option C — no stability promise until 1.0
- **Pros:** freedom.
- **Cons:** contradicts ADR 0024.

## Decision

The shape `ail api` prints from the HIR is versioned by the `api` key:
adding an optional key keeps the version; removing, renaming or changing
the meaning of one bumps it. The in-memory HIR may change freely behind it.
MIR and machine IR are internal; a future `ail mir` dump says in its first
line that it is unstable. Facts computed on MIR are served through the API
as additive keys or reports.

## Consequences

- **Designer, look first at:** whether any MIR view (for performance work)
  should get a stability promise later.
- Generated code (ADR 0228) and computed constant values (ADR 0225) appear as
  additive keys under `api 1`.

## Rejected alternatives

- **Versioned MIR (B):** couples agents to optimiser internals.
- **No promise (C):** breaks ADR 0024.
