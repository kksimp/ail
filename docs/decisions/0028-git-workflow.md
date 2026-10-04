# 0028. Git workflow: `main` is the only long-lived branch

- **Status:** Accepted. Rule 3 (CI-gated pull requests) is **amended (interim)** by [ADR 0085](0085-interim-local-testing-gate.md).
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** Project process
- **Spec rules:** none
- **Resolves:** OQ-024

## Context

The designer asked for work to happen on `main`. [ADR 0027](0027-ci-gating-all-targets.md)
requires all five targets to pass before code lands, and a direct push to
`main` can't be CI-gated. The designer accepts branches only if they stay few
and get folded back into `main` quickly.

## Options considered

### Option A — Docs direct to `main`; code through short-lived, CI-gated branches (chosen)
- **Pros:** The fast doc workflow is kept, and the five-target gate actually
  applies to code.
- **Cons:** A pull-request step for code changes.

### Option B — Always direct to `main`, CI after push
- **Pros:** Fastest.
- **Cons:** `main` can be broken on any target, which defeats ADR 0027.

### Option C — Direct to `main` with a local pre-push check
- **Pros:** No PRs.
- **Cons:** One machine can't run all five targets, so the gate is incomplete.

## Decision

1. **`main` is the only long-lived branch.**
2. **Documentation and specification changes** are committed directly to `main`.
3. **Code changes** (from Stage 2 onward) go through a **short-lived branch**
   and a pull request that must pass the five-target CI gate (ADR 0027).
4. A branch is **merged and deleted as soon as its CI passes**. Branches are
   never left open "for later".
5. Keep the number of open branches to the **minimum needed**. Normally there
   is one branch per change in flight, and never hundreds.
6. Agents must track every branch they create and report it until it's
   merged and deleted.

## Consequences

- AGENTS.md carries these rules for AI contributors.
- Stale branches are a process bug. They should be merged, or deleted with
  the reason recorded.

## Rejected alternatives

- **Always direct to `main`:** breaks the CI gate.
- **Pre-push local checks only:** can't cover all five targets.
