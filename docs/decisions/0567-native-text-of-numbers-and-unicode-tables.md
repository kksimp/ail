# 0567. Native text of numbers reuses the output routines, and character predicates are exact for U+0000 to U+00FF

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

### Option A — text of a float by running the output routine over the free end of the stdout buffer and taking the bytes back; Unicode exact only for U+0000 to U+00FF now, tables in the next slice (chosen)
- **Pros:** No second float formatter to keep identical to the first. The
  integer, `bool` and `char` forms are direct. The common cases (ASCII,
  Latin-1) are exact now, and the compiler's own source needs only ASCII.
- **Cons:** A float's text costs a copy. Above U+00FF the predicates and case
  mappings are approximations (Latin Extended, Greek and Cyrillic mostly
  right, other scripts not), recorded as OQ-172.

### Option B — a second float formatter writing to a string
- **Pros:** No buffer trick.
- **Cons:** Two copies of the shortest-digits algorithm to keep equal.

### Option C — generate the Unicode tables first
- **Pros:** Exact everywhere.
- **Cons:** About 30 KB of generated tables and a generator; delays the whole
  string slice, and no program the compiler builds needs it yet.

## Decision

`rt.str_float` and `rt.str_fixed` run `out_float` and `out_float_fixed` after
making room in the stdout buffer and then cut the text out of it. The `char`
predicates and mappings are exact for U+0000 to U+00FF and approximate above,
until the generated tables of OQ-172 land.

## Rationale

Token usage and AI understanding don't separate the options; speed of delivery
does, and A ships the whole common path with the exactness gap named.

## Consequences

- `ail build` of a program that uses `is_alpha`, `to_upper` or `to_lower` on
  characters above U+00FF may differ from `ail run` until OQ-172 is done.
- Lane 177 may replace `str_float` and `str_fixed` with direct formatters.

## Rejected alternatives

B duplicates the algorithm; C blocks the slice on data the first user does not
need.
