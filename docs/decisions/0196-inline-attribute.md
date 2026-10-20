# 0196. `@inline` is accepted and recorded as a hint

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax
- **Spec rules:** [gram.attr.3](../spec/03-grammar.md)
- **Evidence:** deciding criterion: AI understanding. No syntax is new (`gram.attr.1` already specifies the form).

## Context

The grammar chapter's own example uses `@inline`. The bootstrap rejected it as
an unknown attribute, so the example was skipped. The interpreter has no inliner.

## Options considered

### Option A — accept and ignore (chosen)
- **Pros:** programs written for the optimising backend run now; the example runs.
- **Cons:** the hint does nothing yet.

### Option B — keep rejecting until the backend exists
- **Cons:** forces edits to programs when the backend lands.

### Option C — `@inline` as a requirement, an error when not inlinable
- **Cons:** needs the backend and an inlinability rule that doesn't exist.

## Decision

`@inline` takes no arguments, never changes behaviour, and an implementation may
ignore it. The bootstrap ignores it. Unknown attributes stay errors (`test.attr.1`).

## Consequences

- The optimising backend (ADR 0022) decides what the hint does; `@inline` as an
  obligation would be a later, separate decision.

## Rejected alternatives

- **B** and **C**: see above.
