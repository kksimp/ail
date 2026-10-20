# 0202. Offer fixes for five more common mistakes

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI semantic API
- **Spec rules:** `fix.edit.1`–`fix.edit.5` (non-normative, [semantic-api.md](../design/semantic-api.md)); no change to `docs/spec/`

## Context

Lane 1 added `fixes` to diagnostics ([ADR 0145](0145-diagnostics-schema-v1.md)) but only three diagnostics carried one. An agent that edits a file by hand for `count = 2` on an immutable binding spends tokens a tool can save. The deciding criterion is ADR 0076 #1 (tokens): a fix is one call instead of reading a diagnostic and rewriting a line, and #2 (AI understanding): a fix is checkable.

## Decision

These diagnostics carry a fix (the first of a diagnostic's fixes is the one `ail fix` applies, [ADR 0203](0203-ail-fix-command.md)):

| Rule | Situation | Fix |
|---|---|---|
| `expr.bind.3` | a `:=` binding is assigned or mutated later | insert `mut ` before the name (`fix.edit.1`) |
| `expr.if.1` | `else if` | replace with `elif` (`fix.edit.2`) |
| `err.use.1` | a fallible call as a statement or a `:=` value, in a function whose error type already admits its errors | add `?` at the end of the line (`fix.edit.3`) |
| `expr.match.2` | a `match` misses enum variants, `true`/`false`, or has no `_` | add the arms, each `=> todo` (`fix.edit.4`) |
| `std.use.2` | a standard module's name used without `use std.NAME` | insert the import on line 1 (`fix.edit.5`) |

The match fix makes the program check (with `expr.hole.1` warnings), so an agent can then fill each hole ([ADR 0147](0147-typed-holes-todo.md)).

Unused `mut` and `==` without `impl Eq` get no fix: the checker has no unused-`mut` warning, and the `Eq` fix is more than one line.

## Rationale

Each fix is the single most likely repair and cannot change what a correct program means: adding `mut` or `?` or an arm turns a rejected program into an accepted one, and every fix is re-checked by `ail fix`. The `?` fix is offered only when the function's declared error set already contains the call's errors (or is inferred, `!T`), so it never creates an `err.prop.3`.

## Consequences

- Fixes that need a position the AST doesn't record (the end of a block, the end of a line) use anchored edits ([ADR 0204](0204-anchored-fix-edits.md)).
- `mut` is offered even when the assignment was a typo for another name; `ail fix` is a tool for a reviewed edit, not a proof of intent.

## Rejected alternatives

- **Fix `err.use.1` with `else` handlers.** Needs a choice of handler; `?` is the default the language's own help gives.
- **Replace the match with a `_ => todo` arm.** Hides which variants are missing; one arm per variant keeps the hole typed and the variant visible.
- **Put the `use` after existing imports.** Needs the AST to record them; line 1 is always valid.
