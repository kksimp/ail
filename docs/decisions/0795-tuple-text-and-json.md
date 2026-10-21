# 0795. Tuples as text and JSON: `(1, "a")` and arrays

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076), under the designer's ADR 0791
- **Topic:** 04 — Types; std JSON
- **Spec rules:** `type.tuple.7`, `type.tuple.8`, `json.derive.1-2` ([04-types](../spec/04-types.md), [std-json](../spec/std-json.md))

## Context

ADR 0791 makes `to_str`, `==`, `<`, `hash` elementwise and maps a tuple to a
JSON array both ways. The lane settles the details.

## Decision

- **Text:** `(1, "a")`: elements in their debug form (strings quoted, as in a
  list), comma and space, in parentheses.
- **Order:** lexicographic, element 0 first; a tuple is `Eq`, `Ord` and
  `Hash` exactly when each element is, so `(float, int)` is not a map key.
- **JSON:** derived `ToJson` writes a tuple field as an array of its elements
  (`(int, (bool, str))` gives `[7, [true, "x"]]`), and `T?` of a tuple writes
  `null` for `none`. Derived `FromJson` reads an array of exactly as many
  elements; any other length fails with `expected N elements, found M` under
  the field's path. There is no `Json` conversion for a bare tuple value
  outside a derived type: wrap it in a struct, which is what a schema needs.

## Token measurement

No syntax is added. The derive change is in the prelude generator; a struct
with a tuple field costs what the field costs.

## Rejected alternatives

- **An object `{"0": ..., "1": ...}`:** no other JSON producer writes that,
  and Python's `json` writes arrays.
- **Text without quotes, `(1, a)`:** ambiguous for strings with commas; lists
  already quote.
- **`impl ToJson` for every tuple type:** needs generic impls over arity;
  the derive covers the field case agents write.
