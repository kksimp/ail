# 0134. `char`: traits, classification, digits and code points

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 4 — Type system; 21 — Standard library
- **Spec rules:** [type.char.1–4](../spec/04-types.md), [lex.str.4](../spec/02-lexical.md)
- **Extends:** [ADR 0089](0089-strings.md) (it made `'x'` a `char` literal and left everything else to the library)
- **Evidence:** `examples/interp.ail`, `examples/json.ail`, `examples/sheet.ail`

## Context

ADR 0089 gave `char` a literal and nothing else. The three programs are
scanners: each walks a `List[char]` from `s.chars()` and asks what a character
is (digit, letter, space), converts hex digits (JSON's `\uXXXX`), and goes
between a character and its number (JSON's `\u0007` output). The questions that
the first design left open, and that a Python, Rust or Kotlin author would
each answer differently:

| Question | Spellings in use |
|---|---|
| Is a digit only 0-9? | ASCII: Rust `is_digit(r)`. Unicode `Nd`: Python `isdigit`, Kotlin, Go |
| What is a letter or a space? | Python `isalpha`/`isspace` differ from the Unicode properties |
| Character to number | `ord(c)`, `c as u32`, `c.code`, `c.to_int()` |
| Number to character | `chr(n)`, `char::from_u32(n)`, `n.to_char()` |
| The apostrophe | `'\''` in C-family languages; nothing in ADR 0089 |

## Deciding criterion

Token usage **ties**: the methods are rare in the programs, and no spelling
differs by 2%. **AI understanding** decides. Every method gets one exact
definition, so a call means the same on every platform. The ASCII-only
`is_digit` is the key choice: it is true exactly when `to_digit(10)` returns a
value (and when `s.to_int()` accepts that digit, ADR 0124), so there is no
"it is a digit but it doesn't parse" case. Names are the short ones
(`is_alpha`, `is_space`), not Rust's `is_alphabetic` and `is_whitespace`.

## Decision

1. **The type (type.char.1).** `char` is a Unicode scalar value (type.str.1).
   It has `Eq`, `Ord` (by code point), `Hash`
   ([ADR 0133](0133-automatic-hash.md)) and `ToStr`. Its text form at top
   level (`print`, interpolation, `join`) is the character itself. Inside
   another value it is a char literal in single quotes, with the escapes of
   [ADR 0135](0135-string-literals-and-methods.md).
2. **The literal (lex.str.4).** A char literal is one scalar value or one escape
   of lex.str.1, which now includes `\'`, in single quotes. `'''` is an error
   with the help `'\''`. A char literal never interpolates: `'{'` is `{`.
3. **Classification (type.char.2).**

   | Method | True when |
   |---|---|
   | `is_digit()` | the character is ASCII `0`-`9` only |
   | `is_alpha()` | it has the Unicode `Alphabetic` property |
   | `is_alnum()` | `is_alpha() or is_digit()` (on purpose: ASCII digits only) |
   | `is_space()` | it has the Unicode `White_Space` property |
   | `is_upper()`, `is_lower()` | it has the Unicode `Uppercase`, `Lowercase` property |

   `to_upper()` and `to_lower()` use the single-character mapping. A character
   whose mapping is several characters (`ß`) is unchanged. The Unicode version
   is the implementation's (the bootstrap uses its pinned Rust toolchain's).
4. **Digits (type.char.3).** `c.to_digit(radix) int?` gives the value of `0`-`9`,
   `a`-`z` and `A`-`Z`, worth 0 to 35 (either case). It is `none` when `c` is
   not one of those or its value is not below `radix`. A radix outside 2..36
   traps.
5. **Code points (type.char.4).** `c.code() int` is the scalar value.
   `char.from_code(n) char?` is `none` for surrogates and for values out of
   range. Helps: `ord(c)` says "write `c.code()`", `chr(n)` says "write
   `char.from_code(n)`", and `c.to_int()` says "`c.code()` or `c.to_digit(10)`".

```ail fragment
print('é'.is_alpha())           # true
print('٣'.is_digit())           # false: ASCII only, so is_digit implies to_digit(10)
print('F'.to_digit(16))         # 15
print('g'.to_digit(16))         # none
print('ß'.to_upper())           # 'ß'
print('A'.code())               # 65
print(char.from_code(0xD800))   # none: a surrogate
print(['a', '\''])       # ['a', '\'']
```

## Consequences

- A character and a one-character string are different types, so `c == "a"` is
  a type error. ADR 0135 lets the searching `str` methods take either.
- Results for code points assigned in newer Unicode versions follow the
  toolchain. Pinning a version in the spec can come later.
- The spec lists of built-in types in the `Eq`, `Ord` and `Hash` rules include
  `char`.
- `is_ascii_*`, `is_punct` and similar are not provided until a program needs
  them.

## Rejected alternatives

- **`c.to_int()` and `n.to_char()`:** ambiguous with a digit's value
  (`'7'.to_int()` could be 55 or 7). `code`, `from_code` and `to_digit` each
  say which.
- **A Unicode `is_digit` (Python, Kotlin, Go):** then a "digit" could fail
  `to_digit(10)` and `to_int`, and code that means 0-9 would silently accept
  other scripts' digits.
- **Rust's `is_digit(radix)`:** `to_digit(radix)` already answers it, and a
  second spelling for one question is the AI-understanding cost this ADR avoids.
- **Python's exact classification:** it differs from the Unicode properties.
  Accepted differences: `'Ⅻ'.isalpha()` is `False` in Python but `is_alpha()` is
  true here (U+216B is Alphabetic); `'\x1c'.isspace()` is `True` in Python but
  `is_space()` is false (not White_Space); `'٣'.isalnum()` is `True` in Python
  but `is_alnum()` is false; `'ß'.upper()` is `'SS'` in Python and `ß` here.
- **Full (multi-character) case mapping:** `to_upper()` would have to return a
  `str`, and a method that changes type with its input is harder to use.
