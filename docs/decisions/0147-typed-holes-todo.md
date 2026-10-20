# 0147. Typed holes: the keyword `todo`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax; 20 — AI semantic API
- **Spec rules:** [expr.hole.1–2](../spec/05-expressions.md), [lex.kw.1](../spec/02-lexical.md)
- **Amends:** [ADR 0114](0114-reserved-keywords.md) (adds `todo` to the reserved words)
- **Evidence:** `tools/tokbench/corpus/holes/`
- **Plan:** [ai-first-plan.md](../design/ai-first-plan.md) §2.5

## Context

An AI writes a program top-down and wants the compiler to say what each gap
needs (wishlist 2.2). A hole must be an expression of any type that the
checker accepts with a warning and that traps if reached.

## Options considered

Measured with `tools/tokbench/run.sh tools/tokbench/corpus/holes/*.ail`:
three example programs with 3–4 bindings' values replaced by a hole.

| Spelling | commands | shapes | sheet | Note |
|---|---|---|---|---|
| **`todo` (chosen)** | 504.8 | 424.4 | 4784.6 | a word every model knows |
| `hole` | 504.8 | 424.4 | 4784.6 | jargon outside type theory |
| `???` | 505.6 | 425.0 | 4785.4 | +1 token per hole on one tokenizer; clashes visually with `?` (err.prop.1) |

(Mean tokens over five tokenizers.)

## Decision

1. `todo` is a reserved keyword and an expression (amends ADR 0114).
2. The checker gives a hole the expected type, or an unknown type that every
   use accepts, and reports warning `expr.hole.1` stating the expected type
   and the local names in scope with their types, sorted.
3. Evaluating a hole traps with `expr.hole.2`.

## Rationale

**Tokens tie** (under 0.2%). **AI understanding** decides: `todo` reads as
"not written yet" to every model and human, and it doesn't look like the `?`
operator. `???` would also need a new lexer symbol.

## Consequences

- `todo` can no longer be a name. No example or test used it.
- The bootstrap doesn't infer an unknown hole's type from its later uses;
  locals bound to such a hole have an unknown type and aren't checked.

## Rejected alternatives

- `hole`: same cost, less familiar.
- `???`: costs more on one tokenizer, a new symbol, and confusable with `?`.
- A prelude function `todo()`: `()` costs a token each time, and a function
  can't take any type without generics over the return type.
