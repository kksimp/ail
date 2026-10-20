# 0346. The formatter splits long lines before binary operators

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax
- **Spec rules:** `lex.stmt.2`, `fmt.op`

## Context

After list splitting ([ADR 0169](0169-canonical-formatting-style.md)) and chain splitting ([ADR 0323](0323-chained-call-lines.md)), a long `if a and b and c` or `x + y + z` condition still stays on one line. lex.stmt.2 continues a statement only after a trailing operator, inside brackets, or before `.name`.

## Decision

1. lex.stmt.2 gains a fifth case: a newline does not end a statement when the next line's first token is `and`, `or`, `+`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>` or `>=`. None of these can start a statement. `-` is excluded because it starts a negation; `+=`, `*=`, `/=` and `%=` are excluded so their existing diagnostics hold.
2. fmt.op: a line still over 100 columns after fmt.wrap and fmt.chain is split before each top-level operator of the lowest precedence, one extra indent, operator first. `-` ends its line instead. `<` and `>` are never split points (type arguments). A line with a top-level closure is not split. Pieces lay out again recursively.

## Rationale

**Tokens decide, and tie.** Only newlines and indentation move; the corpus is unchanged (0.00% token delta, under the 0.5% limit). **AI understanding decides** the shape: the operator first shows each clause's role at the start of its line, and diffs touch one clause.

## Rejected alternatives

- *Trailing operators for all.* Needs no lexer change but hides the operator that matters at the far end of a line.
- *Wrapping in parentheses.* Costs two tokens and makes the formatter add brackets, which `fmt.safe` forbids.
- *Leading `-`.* A line `-x` is a legal statement, so continuing it would change meaning.
