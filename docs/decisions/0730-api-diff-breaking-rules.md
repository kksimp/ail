# 0730. What `ail api --diff` marks as breaking

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 20 — AI semantic API
- **Spec rules:** api.diff.3
- **Builds on:** [ADR 0729](0729-api-diff-command.md)

## Context

The point of the command is the questions "will my caller still compile" and
"can an implementer still work". The marker must be cheap to read and rarely
wrong.

## Decision

`breaking:` marks: a removed declaration; any signature change; a new effect
or error type; a new or changed `requires`; an `ensures` that went away; a
trait method that lost its default; an added enum variant (exhaustive
`match` breaks); an added trait method with no default. Everything else, such
as an added function, a dropped effect, a dropped `requires` or a new
`ensures`, is not breaking.

Parameter-name changes count as signature changes, because AIL has named
arguments (ADR 0454).

## Rejected alternatives

- **Type-aware compatibility (widening a parameter is fine).** It needs
  subtyping rules the checker doesn't expose and risks unsound "safe" claims.
  A signature change is reported and the reader judges.
- **Mark nothing, only list.** Agents would have to re-derive the rules.
