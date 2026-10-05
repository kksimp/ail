# 0124. `s.split(sep)` and `s.to_int()`

- **Status:** Accepted
- **Date:** 2026-10-05
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 21 — Standard library
- **Spec rules:** [type.str.4–5](../spec/04-types.md)

## Context

Parsing text, the next milestone's program, needs two string functions
before the standard library is designed.

## Deciding criterion

Tokens tie. **AI understanding** decides on the most common names and
behaviour:
- `split` behaves as in Python, JavaScript and Go: every separator splits,
  and empty pieces are kept.
- `to_int` returns an optional (Swift's `Int(s)`, Kotlin's
  `toIntOrNull`). Bad input then has a single, obvious outcome, and `else`
  handles it in one line.

## Decision

1. `s.split(sep)` returns a `List[str]` of the pieces of `s` between
   occurrences of `sep`. Empty pieces are kept, so `n` separators always
   give `n + 1` pieces. An empty `sep` traps.
2. `s.to_int()` returns an `int?`: the value of `s` when `s` is an
   optional `-` followed by one or more decimal digits and fits in `int`.
   Otherwise it returns `none`.

```ail
qty := words[2].to_int() else return BadQuantity(text: words[2])
```

## Rejected alternatives

- **`parse_int` returning an error type:** that needs a built-in error
  type, and most callers replace it with their own error anyway.
- **`split` dropping empty pieces:** loses information (`"a,,b"`), and
  differs from the languages AIs know best.
