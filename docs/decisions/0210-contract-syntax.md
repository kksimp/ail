# 0210. Contracts are `requires` and `ensures` lines under the signature

- **Status:** Accepted; amends [ADR 0114](0114-reserved-keywords.md) (adds `requires` and `ensures`)
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax
- **Spec rules:** [contract.decl.1–2, contract.ens.2](../spec/20-contracts.md), [lex.kw.1](../spec/02-lexical.md)

## Context

Wishlist 6.1 of the AI-first plan: an AI states intent, and the toolchain
checks the code against it. Preconditions and postconditions are the
smallest useful form. Tonight they are checked at run time; the syntax must
also suit a later static prover. [ADR 0083](0083-attribute-syntax.md) makes
attributes same-line metadata, and a contract changes what a program does.

## Measurement

`tools/tokbench/run.sh` on three programs (arithmetic, a bank account with
a method-like struct, a list search; 12 conditions), mean of the five
offline tokenizers:

| Form | Tokens | vs. no contracts |
|---|---|---|
| No contracts | 351.4 | — |
| `requires`/`ensures` lines (chosen) | 482.8 | +11.0 per condition |
| `@requires(...)` attributes on the `fn` line | 499.8 | +12.4 per condition |
| `pre`/`post` lines | 482.8 | same as chosen |

## Options considered

### Option A — keyword lines under the signature (chosen)
```
fn divide(a int, b int) int
    requires b != 0
    ensures result * b == a
    a / b
```
- **Pros:** 3.5% fewer tokens than attributes; one condition per line, so a
  diff or a trap names one line; reads as part of the signature, like Eiffel,
  Dafny and Ada 2012; a semantic construct gets a keyword, as ADR 0083 asks.
- **Cons:** two new reserved words.

### Option B — `@requires(cond)` attributes
- **Pros:** no new keywords.
- **Cons:** 3.5% more tokens (the `@`, parentheses and repeated names);
  long conditions push the `fn` line past the 100-column limit; ADR 0083
  keeps attributes for metadata.

### Option C — `pre`/`post`
- **Pros:** shorter words.
- **Cons:** the same tokens as `requires`/`ensures` in every tokenizer
  measured; `pre` and `post` are common names, and the longer words say
  what they do.

## Decision

A function's contract is zero or more `requires COND` and `ensures COND`
lines directly under its signature, before the body's first statement.
`requires` and `ensures` are reserved (amending ADR 0114). In `ensures`,
`result` names the success value. `result` is **not** reserved: it is a
name the `ensures` lines bind, and a function with an `ensures` line can't
also have a parameter called `result`.

## Rationale

Tokens decide between A and B (ADR 0076 criterion 1); readability decides
between A and C, which tie on tokens. Reserving `result` would break every
program that uses it as a variable (three of the repository's examples and
the token corpus do), and `result` is one of the names AI writers produce
most often; a reserved `result` would turn a habit into a compile error.
Binding it only in `ensures` keeps ADR 0114's rule that a word means one
thing: in an `ensures` line `result` is an ordinary name with a known
value, like a parameter.

## Consequences

- `requires` and `ensures` can no longer be used as names.
- `old(x)` (a parameter's value on entry) is out of scope tonight; an
  `ensures` line sees parameters' values at return. OQ-060.
- The formatter needs no change: contract lines are ordinary body lines.

## Rejected alternatives

- **Attributes:** more tokens, and against ADR 0083's split of metadata
  from semantics.
- **`pre`/`post`:** no token saving, worse readability.
- **Reserving `result`:** breaks common code for no gain in clarity.
- **A `where` clause on the signature line:** the same long-line problem
  as attributes.
