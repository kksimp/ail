# 0323. A line starting with `.name` continues the statement; the formatter splits chains one call per line

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax
- **Spec rules:** `lex.stmt.2`

## Context

`ail fmt` splits a long line at its longest list ([ADR 0169](0169-canonical-formatting-style.md)), but a chain such as `xs.filter(f).map(g).sorted()` has no list to split at, and lex.stmt.2 allows a continuation only inside brackets, or after a comma or binary operator. Agents write long chains, and one-per-line diffs are smaller.

## Decision

1. lex.stmt.2 gains a fourth case: a newline does not end a statement when the next line's first token is `.` followed by a name. The lexer takes the newline back.
2. The formatter (fmt.chain) splits a line over 100 columns with two or more `.method(...)` calls at its top level (and no operator after the first) into the head and one `.method(...)` per line, indented one level. A chain that fits is joined onto one line. Anything else keeps fmt.wrap.

## Rationale

**Tokens decide, and tie.** The change moves only newlines and indentation, which the tokenizers merge with neighbouring whitespace. Every file under `examples/`, `tools/edit/` and `bootstrap/tests/cases/` was already within the limit or has no chain, so formatting them changes 0 tokens (0.00%, under the 1% budget). **AI understanding decides** the shape: a leading `.` is unambiguous (no statement starts with `.`; `..` is excluded), and one call per line makes a diff touch one call.

## Rejected alternatives

- *Trailing `.` at the line end.* Rejected: a dot at the end hides the next name from the line it belongs to, and needs another continuation rule.
- *Splitting only inside brackets (wrap the chain in parentheses).* Rejected: costs two tokens per chain and makes the formatter add brackets, which `fmt.safe` forbids.
- *Splitting any line at an operator.* Done in [ADR 0346](0346-formatter-splits-at-binary-operators.md).
