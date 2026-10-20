# 0372. A name is letters, ASCII digits and `_`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 2 — Lexical structure
- **Spec rules:** `lex.name.1`

## Context

The spec left identifiers undesigned (chapter 02). The bootstrap's lexer
continued a name with Rust's `char::is_alphanumeric`, which includes every
Unicode numeric character (`²`, `٣`, `Ⅷ`). AIL's own `char.is_alnum()` is
`is_alpha()` or an ASCII digit (`type.char.2`), so the lexer written in
AIL (ADR 0371) split `x٣` where the Rust one didn't. Fuzzing the two found
it.

## Options considered

### Option A — a name is Unicode letters, ASCII digits and `_` (chosen)
It starts with a letter (`Alphabetic`) or `_`, and continues with letters,
ASCII digits or `_`: exactly `is_alpha()` and `is_alnum()`.
- **Pros:** one definition in the language and in its library; digits in
  names are the ones `to_digit` reads; no look-alike numerals in names.
- **Cons:** a name like `x²` that the bootstrap accepted is now an error
  (`bootstrap.unsupported` at the `²`); no program in the repository has one.

### Option B — keep Rust's rule and add `char.is_numeric()` to AIL
- **Cons:** a second definition of "digit" in the library, kept only to
  allow superscripts and other numerals in names.

## Decision

Option A (`lex.name.1`).

## Rationale

AI understanding and readability (ADR 0076 criteria 2–3): one meaning of
"name character", the same in the language and its library. The cost falls
on a construct no program uses.

## Consequences

- The Rust and AIL lexers agree on every name.
- Normalisation and confusable characters stay undesigned (chapter 02).

## Rejected alternatives

- **B:** adds library surface to keep a rare and confusing spelling.
