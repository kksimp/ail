# 0264. Fixes that rewrite a call, a bracket form or a chained comparison are anchored to the source line

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** AI semantic API (fixes)
- **Spec rules:** `gram.foreign.1`, `gram.help.2`; extends ADR 0145 (diagnostics schema), ADR 0203 (`ail fix`)

## Context

Many of the common mistakes are rewrites of an expression: `len(xs)` becomes `xs.len()`, `str(n)` becomes `"{n}"`, `range(3)` becomes `0..3`, `print(a, b)` becomes `print("{a} {b}")`, `Point(1, 2)` becomes `Point(x: 1, y: 2)`, `0 < n < 5` becomes `0 < n and n < 5`, `Option[int]` becomes `int?`. The syntax tree has start positions only, so the checker can't name the end of an argument.

## Options considered

### Option A — anchors resolved against the source line (chosen)
`Edit` already has anchors that the report step resolves against the text (`old`, `AfterBlock`, `LineEnd`). Add `Call` (`callee(args)` and a template), `Brackets` (`callee[args]`), `CallNamed` (prefix each argument with its field name), `Chain` (repeat the middle operand with `and`) and `TrimBefore` (delete a character and the blanks before it). Templates use `$1 $2` (arguments as written), `$r1` (the first as a method receiver, parenthesised unless it is a plain postfix expression), `$a` (all arguments) and `$*` (all as one interpolated string).
- **Pros:** no change to the syntax tree or the shared parser and checker structure; the resolved edit is an ordinary text edit in schema v1, so agents and `ail fix` see no difference.
- **Cons:** a call that spans lines isn't rewritten (the fix is dropped, never wrong); the anchor reads one line.

### Option B — end positions in the syntax tree
- **Pros:** exact for any expression.
- **Cons:** edits every AST constructor and parser site that other lanes are changing tonight.

## Decision

Option A. A fix whose anchor can't be resolved is dropped, as for the existing anchors. Strings with quotes in arguments are not interpolated (`str("a")` gets help but no fix). Fixes that change meaning aren't offered: `int(x)` has help only, because the right method depends on the type of `x`.

## Rationale

Tokens and AI understanding are served by a fix that applies verbatim; exactness for multi-line calls is rare in the corpus (no case).

## Consequences

- 74 of the 109 corpus programs are resolved by `ail fix`; the remaining 35 need a decision (`try`, `switch`, `a ? b : c`, wrong argument types, a missing `main`).
- The merged fixes of one line (`gram.rec.1`) require explicit edits; anchored ones aren't merged.

## Rejected alternatives

- **A second tokenizer in the fix step:** duplicates the lexer.
- **Offer `int(x)` as `x.to_int()`:** wrong for `str` (gives `int?`) and silently changes the type.
