# 0146. `ail explain RULE` prints a rule from the embedded specification

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API ([ADR 0024](0024-semantic-api-from-stage-2.md))
- **Spec rules:** none (tool behaviour; documented in [semantic-api.md](../design/semantic-api.md))
- **Plan:** [ai-first-plan.md](../design/ai-first-plan.md) §2.2

## Context

Every diagnostic cites a rule ID (ADR 0002). To understand one, an agent had
to search `docs/spec/`, which costs tokens and needs the repository.

## Options considered

### Option A — embed the spec in the binary at build time (chosen)
A build script includes every `docs/spec/*.md` and the ADR index.
- **Pros:** works anywhere the binary is; can't drift from the spec it was
  built with; new chapters need no code change.
- **Cons:** about 125 KB more binary.

### Option B — read `docs/spec/` at run time
- **Cons:** breaks when the binary is copied elsewhere; the path must be found.

### Option C — a hand-written explanation table in the checker
- **Cons:** a second copy of the spec that drifts.

## Decision

1. `ail explain RULE` prints the rule's ID and location, its text (to the
   next rule, heading or code block), each cited ADR with its title, and the
   nearest following `ail` example in the same section.
2. `--json` prints one schema v1 object:
   `{"api":1,"rule","file","line","text","adrs":[{"id","title"}],"example"?}`.
3. An unknown rule exits 2 with `error: unknown rule ...` and a help listing
   known rules nearby (same section, else one per section of the chapter).
4. `bootstrap.unsupported` is explained as the implementation-limit code.

## Rationale

**AI understanding** decides: the explanation is one command away and exactly
matches the binary's spec. Tokens favour it too: one rule instead of a
chapter. Runtime cost is nil (the command only reads embedded text).

## Consequences

- A typical fix per rule (plan §2.2) isn't in the spec yet; the checker's
  `fixes` cover the common cases. Adding a "Fix:" line to rules would let
  `explain` show it.

## Rejected alternatives

- Run-time reading (B): fragile outside the repository.
- Hand-written table (C): duplicates normative text.
