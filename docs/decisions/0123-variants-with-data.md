# 0123. Variants with data are built with named fields and matched by position

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system
- **Spec rules:** [type.enum.6](../spec/04-types.md), [expr.match.4](../spec/05-expressions.md), [expr.match.6](../spec/05-expressions.md)

## Context

ADR 0053 declares variants with data (`Circle(r f64)`), and the spec's
`match` example binds them by position (`Rect(w, h) => w * h`). How to
build them wasn't decided.

## Deciding criterion

**Token usage** is close to a tie: naming the fields costs about two tokens
per field on construction, and constructions are rarer than uses. **AI
understanding** decides: building a variant follows the struct literal
rule (ADR 0058, ADR 0113), so there is one rule for every literal with fields,
and arguments can't be swapped by accident. Patterns stay positional as
specified, so the common `match` stays short.

## Decision

1. A variant with data is built like a struct literal, naming every field
   once in any order: `BadNumber(text: s)`, or `ParseError.BadNumber(text: s)`
   where the type isn't known.
2. A pattern `Variant(a, b)` binds the variant's fields, in declaration
   order, to new immutable names. `_` skips a field. The names follow the
   no-shadowing rule.

## Rejected alternatives

- **Positional construction** (`Circle(1.0)`): two rules for literals, and
  arguments can be swapped silently.
- **Named patterns** (`Circle(r: radius)`): more tokens in every `match`.
