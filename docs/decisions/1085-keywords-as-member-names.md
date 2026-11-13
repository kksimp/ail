# 1085. Keywords as field, member, label and method names

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer approved every item of the AI-first language review for 0.1 on 2026-11-12, item 20 among them)
- **Topic:** 2 — Core syntax
- **Spec rules:** [lex.kw.1, lex.kw.3](../spec/02-lexical.md)
- **Builds on:** [ADR 1067](1067-closures.md) (`xs.any(f)` after `.`)
- **Amends:** [ADR 0114](0114-reserved-keywords.md) (keywords were reserved in every position)

## Context

Agents name fields after what they hold: `error`, `match`, `in`, `ref`,
`any`, `todo`. Every keyword was reserved everywhere, so a struct with an
`error str` field failed at the field, at each `Outcome(error: "")` and at
each `o.error`: six errors in one review program, none with a fix. In those
positions a keyword can mean nothing else, so the reservation bought no
clarity there. ADR 1067 had already let `any` follow `.` to name the list
method.

## Decision

A keyword is a plain name (`lex.kw.3`):

1. as a field of a struct or of an enum variant (`error str`,
   `Failed(error str)`), except `pub`, `fn` and `invariant`, which begin a
   member line;
2. after `.` (`o.error`, `xs.any(f)`, `t.match()`);
3. as a label before `:` in a call or literal (`Outcome(error: "")`);
4. as a method name: `fn T.any(self)`, and `fn error(self)` inside a `trait`
   or `impl` block;
5. after `uses` in `impl Hash`.

Inside a method a keyword-named field is reached as `self.error`; the bare
word stays the keyword. Patterns bind fields by position, so `Failed(e)`
needs nothing new. Everywhere else `lex.kw.1` holds. Its help now names a
replacement (`error` -> `err`, `todo` -> `pending`, `any` -> `found`, ...).
For words that have no other use inside a function body (`error`, `todo`,
`any`, `ref`, `enum`, `struct`, `use`, `pub`) the error carries a fix that
renames the binding and each later use of the word in the function: an
occurrence not after `.` and followed by a symbol other than `(`, a line end,
`and`, `or`, `else` or `in`, inside interpolations too. The formatter spaces a
keyword in these positions as a name. Derived `ToJson` binds variant fields as
`v_NAME`, so a keyword field derives.

## Options considered

### Option A — keywords are names in member positions (chosen)
- **Pros:** the positions are unambiguous to a reader and to the parser (a
  `.` before, a `:` after, a field line); the program an agent writes first
  checks; no new syntax to learn.
- **Cons:** the same word is a keyword in one place and a name in another;
  `self.error` is required where a plain field is reachable bare.

### Option B — keep every keyword reserved, better fix only
- **Pros:** one rule, no context.
- **Cons:** the agent still has to rename the field everywhere, and JSON or
  API field names such as `error`, `match`, `in` can't be mirrored by a struct
  field without a mapping layer.

### Option C — an escape such as `r#error` or `` `error` ``
- **Pros:** works in every position.
- **Cons:** foreign spelling agents rarely produce unprompted; more tokens on
  every use; a second spelling of every name.

### Option D — contextual keywords (reserve fewer words)
- **Pros:** fewer reserved words overall.
- **Cons:** reopens the keyword list of ADR 0114 for each word and makes
  statement starts ambiguous (`match x` vs a variable `match`); larger change
  for the same programs.

## Consequences

- The review program `p4_pool.ail` gets past its six keyword errors.
- `lex.kw.1` becomes fixable (`compiler/explain/fixable.txt`).
- A top-level `fn any()` is still refused: nothing could call it.
