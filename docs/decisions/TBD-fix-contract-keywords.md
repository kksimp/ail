# ADR TBD: `ail fix` for misplaced contract keywords

- Status: Accepted
- Decided by: AI (delegated, ADR 0076)
- Date: 2026-10-21

## Context

Two mistakes in the corpus have one correct rewrite, but the checker only
described it: `requires result >= 0` (contract.ens.2) and `requires old(x)`
(contract.ens.3).

## Decision

- contract.ens.2 replaces the `requires` keyword with `ensures`. `result`
  exists only after the call, so the line was about the result.
- contract.ens.3 in a `requires` drops the `old(...)` call: on entry `old(e)`
  is `e`. The argument is parenthesised unless it is a single name or call
  (new template token `$p`). In any other place (a body, an `invariant`) there
  is no single rewrite, so no fix is offered.
- Both edits are dropped, not guessed, when the source text isn't where the
  checker expects it (fix.edit.2).

## Rejected alternatives

- Fixing a mixed `requires n >= 0 and result >= 0` by moving the line: the
  `n >= 0` part would stop being a precondition. The fix is applied only to
  the keyword; the check stays an error report the author reads first.
- Fixing `old(...)` outside contracts: the right value to capture is a choice.
