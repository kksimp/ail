# 0567. Native text of numbers reuses the output routines, and character predicates use generated Unicode tables

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** native backend (lane 172, native strings and chars)
- **Spec rules:** type.str.3, type.str.8, type.str.11, type.num.7, type.char.2, native.str.1

## Context

Interpolation as a value needs the text of an `int`, `float`, `bool` and
`char` as a `str`. The float algorithm (ADR 0409) writes straight into the
stdout buffer; lane 177 owns number formatting and may replace it. The `char`
predicates and case mappings (`is_alpha`, `to_upper`...) follow Unicode
properties, and the full tables are about 30 KB.

## Options considered

### Option A — text of a float by running the output routine over the free end of the stdout buffer and taking the bytes back (chosen)
- **Pros:** No second float formatter to keep identical to the first. The
  integer, `bool` and `char` forms are direct. The common cases (ASCII,
  Latin-1) are exact now, and the compiler's own source needs only ASCII.
- **Cons:** A float's text costs a copy.

### Option B — a second float formatter writing to a string
- **Pros:** No buffer trick.
- **Cons:** Two copies of the shortest-digits algorithm to keep equal.

### Option C — hand-written approximations of the tables
- **Pros:** Small and quick. Slice 1 shipped this (exact for U+0000 to U+00FF).
- **Cons:** Silently different from `ail run` above U+00FF (a differential run
  over 195,000 scalar values found over 6,000 differences). Replaced in slice 2.

### Option D — range tables generated from the interpreter (chosen for the character data)
- **Pros:** Exact by construction; regenerated with one command when the pinned
  Rust moves. 19 KB of string constants in the runtime (hello world grows from
  34 KB to 54 KB), binary-searched, with the ASCII cases inline.
- **Cons:** Data in every executable until the linker drops unused runtime
  data.

## Decision

`rt.str_float` and `rt.str_fixed` run `out_float` and `out_float_fixed` after
making room in the stdout buffer and then cut the text out of it. The `char`
predicates (`is_alpha`, `is_upper`, `is_lower`) and the single-character case
mappings read tables that `tools/native/gen_unicode.ail` generates from the
interpreter into `compiler/rt/native/unicode.ail`, along with the characters
whose `str` case mapping is longer (`"ß".to_upper()` is `"SS"`). Capital sigma
follows the final-sigma rule, with a small case-ignorable set.

## Rationale

Token usage and AI understanding don't separate the options; speed of delivery
does, and A ships the whole common path with the exactness gap named.

## Consequences

- The native predicates and mappings equal the interpreter's on every scalar
  value the generator saw (checked on U+0080 to U+2FFFF and U+E0000 to
  U+E01FF, on both targets).
- Final sigma treats only apostrophes, dots, accents and combining marks as
  skippable; a rarer case-ignorable character before or after a sigma could
  differ.
- Lane 177 may replace `str_float` and `str_fixed` with direct formatters.

## Rejected alternatives

B duplicates the algorithm; C is silently wrong.
