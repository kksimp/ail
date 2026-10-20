# 0208. `s.len()` counts characters and `s.byte_len()` counts bytes (resolves OQ-043) (*delegated*)

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Strings
- **Spec rules:** `type.str.2`, `type.str.12`

## Context

`str` had no `len()` (ADR 0089); the spelling was `s.chars().len()`, which
builds a list to count it. OQ-043 asked what `len()` should count.

## Options considered

### Option A — `len()` counts characters (Unicode scalar values); `byte_len()` counts bytes (chosen)
- **Pros:** agrees with `s.chars().len()` and with `pad_left(width)`, which
  already count characters (`type.str.9`); no result depends on the encoding;
  `len` is the name an agent reaches for first.
- **Cons:** O(n), so `len()` in a loop is quadratic; Go/Rust users expect bytes.

### Option B — `len()` counts bytes
- **Pros:** O(1).
- **Cons:** `"é".len()` is 2, which disagrees with every other character-counting
  method and surprises Python and JavaScript users, the likeliest writers.

### Option C — no `len()`; named views only
- **Pros:** the cost is visible.
- **Cons:** a longer spelling for the commonest question, and it is exactly
  the case `expr.name.2` was patching with a help message.

## Decision

`s.len()` returns the number of characters (scalar values); `s.byte_len()`
the number of UTF-8 bytes. Grapheme clusters are not counted:
`"e\u{301}".len()` is 2.

## Rationale

AI understanding decided (no syntax is at stake; as a side effect `s.len()`
drops the `.chars()` call from the commonest string question).
The O(n) cost is documented; `byte_len()` is the O(1) route when only the
byte size matters.

## Consequences

- The diagnostic for `s.len()` is gone; `length`, `size` and `count` now
  suggest `len()` / `byte_len()`.
- Bytes as a view (`s.bytes()`) stays unimplemented.

## Rejected alternatives

- Option B: inconsistent with the other character methods.
- Option C: more tokens and a permanent patch.
