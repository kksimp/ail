# 0089. Strings: double-quoted UTF-8 literals with `{expr}` interpolation; no direct indexing

- **Status:** Accepted; amended by [ADR 0135](0135-string-literals-and-methods.md) (exact multi-line string rules, raw strings, `\'`)
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 2 — Core syntax
- **Spec rules:** [lex.str.1–5](../spec/02-lexical.md), [type.str.1–2](../spec/04-types.md)
- **Evidence:** `tools/tokbench/corpus/r4/` interpolation 17.0, `format(...)` 18.4, concatenation 23.2

## Deciding criterion

**Token usage:** interpolation is the cheapest way to build text (−27% vs.
concatenation). **AI understanding** decides indexing: `s[i]` is ambiguous
for UTF-8 text (byte or character?), so it doesn't exist. Code says which one
it means.

## Decision

1. String literals are written in double quotes. Escapes are `\n \t \r \0 \\ \"`,
   `\{`, and `\u{hex}`.
2. `{expression}` inside a string literal interpolates the expression's text
   form. A literal brace is written `\{`. (`}` alone needs no escape.)
3. `"""` begins a multi-line string literal. Its common leading indentation
   is removed.
4. `'x'` is a character literal of type `char`, a Unicode scalar value.
5. `str` is immutable UTF-8 text (a value, ADR 0060). It has **no** direct
   indexing. Code uses explicit views such as `s.bytes` (bytes, indexable)
   and `s.chars()` (characters). Exact APIs belong to the standard library.

```
msg := "Hello, {name}! You have {count} new messages."
```

## Consequences

- `tools/tokbench/corpus/current/tokenize.ail` still uses `src[i]`, which
  predates this ADR. It will be updated when the byte and character APIs
  are designed.
- Raw string literals (no escapes) may be added later for regex and paths.

## Rejected alternatives

- Concatenation (`"a" + x`): 23.2 tokens, against 17.0 for interpolation. Rejected on token usage.
- `format(...)` calls: 18.4 tokens, against 17.0 for interpolation. Rejected on token usage.
- Direct indexing `s[i]`: rejected on AI understanding, because for UTF-8 text it is ambiguous whether `i` counts bytes or characters. Explicit views `s.bytes` and `s.chars()` make the meaning visible.
