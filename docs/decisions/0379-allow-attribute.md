# 0379. `@allow("rule")` before a function allows a named warning in it

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 2 — Core syntax; 4 — Type system
- **Spec rules:** [gram.attr.5](../spec/03-grammar.md), [expr.match.3](../spec/05-expressions.md)
- **Settles:** the suppression syntax that ADR 0056 left open

## Context

ADR 0056 (designer) makes a `_` arm that covers enum variants a warning
(`expr.match.3`) that "can be suppressed per match", with the syntax left
open. The AIL checker in `compiler/check/` matches on `Ty` (21 variants) and
`ast.Expr` (25) in 118 functions, many as predicates (`_ => false`), and
`ail check compiler` printed 167 warnings. The self-hosted front end should
build warning-free.

## Options considered

Measured on `compiler/check/` (302 KB).

### Option A — `@allow("expr.match.3")` before the function (chosen)
An attribute (ADR 0083 already lists suppressing warnings as an attribute's
job) that names the warning's rule. It covers every match in the function.
- **Pros:** about 3 KB for the 118 functions; one mechanism for any warning
  later allowed; the rule ID in the source says what is allowed and where
  it is specified.
- **Cons:** coarser than one match; a function with an `_` arm that should
  list its variants is silent too.

### Option B — list the variants in every arm (`A | B | C => ...`)
- **Cons:** 22.5 KB more (7.5%), 154 lines over 120 columns; a predicate on
  `Ty` names 20 variants to say "anything else".

### Option C — a marker on the arm itself (for example `_! =>`)
- **Pros:** exactly per match, as ADR 0056 words it.
- **Cons:** a new token with no other use, which an agent must learn; it
  can't name which warning it allows.

## Decision

Option A. `@allow` takes one string literal, the rule ID of a warning the
function may allow. The bootstrap allows `expr.match.3` only; any other
argument, or none, is an error (`gram.attr.5`). The warning's help now
mentions the attribute.

## Rationale

ADR 0076 criterion 1 (tokens): Option A costs about a seventh of Option B.
Criterion 2 (AI understanding): it reuses the attribute mechanism and names
the rule, so `ail explain expr.match.3` is one step away. The function is
the unit an agent reads and edits, so allowing per function loses little.

## Consequences

- `ail check compiler` is warning-free.
- `pub` can't precede an attribute (`pub @allow(...) fn` is `mod.vis.1`), so
  helpers that needed both lost `pub`; they were used only inside their
  module.

## Rejected alternatives

Options B and C, above. A file-wide allow: too far from the match it
silences.
