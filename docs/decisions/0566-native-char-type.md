# 0566. `char` is a MIR type held in a register as an `int`, and `chars()` gives `List[char]`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** native backend (lane 172, native strings and chars)
- **Spec rules:** type.char.1-4, type.str.7, native.subset.1, native.str.1

## Context

The first native subset had no `char`. Its runtime made `s.chars()` a list of
one-character strings, which can't be compared with a `char` literal and
prints differently from the interpreter's `['a', 'b']`. The compiler's own
source uses `char` in about 25 places (`s.chars()`, `c.is_digit()`,
`c == '\n'`).

## Options considered

### Option A — a new MIR type `Char`, an `int` in registers and list slots (chosen)
- **Pros:** `c == 'a'`, `<`, `c.code()`, `List[char]` and `for c in s.chars()`
  cost what an `int` costs: one register, 8 bytes in a list. Printing and
  `to_str` know the type, so `print(c)` writes the character and
  `print(cs)` writes `['a', 'b']`.
- **Cons:** One more case in a few `match`es (`Ty.name`, `is_number`, the
  verifier treats it as an `int`).

### Option B — `char` as a one-character `str`
- **Pros:** No new type.
- **Cons:** Every `chars()` allocates a string per character and comparison
  goes through `str_eq`; `c.code()` decodes. About 25 uses in the compiler,
  most in the lexer's inner loop.

### Option C — `char` as plain `int`
- **Pros:** Smallest change.
- **Cons:** `print(c)` would print a number; `to_str` and the interpolation of
  a `char` would be wrong.

## Decision

`mir.Ty` gains `Char`. A `char` value is its Unicode scalar value in a
register, `List[char]` is a list of 8-byte slots, and `s.chars()` returns it
(replacing the list of one-character strings).

## Rationale

Option A is faster than B on the compiler's hottest string loops and, unlike C,
keeps the interpreter's text forms (token usage and AI understanding agree: the
source reads the same in both tiers).

## Consequences

- `c.code()` is a `copy` to an `int` value; `char.from_code` and
  `c.to_digit(radix)` return optionals and wait for lane 175.
- The runtime functions take and return `int` for a character; the lowering
  types them.
- Printing a `List[char]` or a `List[str]` escapes as trait.auto.2 does.

## Rejected alternatives

B allocates in the lexer's inner loop; C prints wrong text.
