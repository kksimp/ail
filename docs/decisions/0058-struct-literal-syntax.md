# 0058. Struct values are constructed `Type(field: value, ...)`; `name: value` is the form for named arguments

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 4 — Type system
- **Spec rules:** [type.struct.3](../spec/04-types.md)
- **Evidence:** cl100k tokens for a 3-field literal: `User(id: 7, …)` 18, with `=` 18, braces 18, no separator 15

## Options considered

- **`User(id: 7, name: "Ada")` (chosen):** the same `name: value` form as
  named arguments, clearly separated, and reads naturally. Con: +3 tokens over the bare form.
- **`User(id 7, name "Ada")`:** the fewest tokens, but reads like function
  application and is ambiguous with expression values.
- **`User{id: 7, …}`:** introduces braces into a brace-free language.

## Decision

1. A struct value is constructed by writing the type name followed by a
   parenthesised list of `field: value` pairs.
2. `name: value` is the form used for named arguments in calls, wherever
   named arguments exist. *Whether* and *when* named arguments are allowed
   or required is still open ([ai-author-wishlist.md 1.5](../design/ai-author-wishlist.md)).

```
u := User(id: 7, name: "Ada", email: none)
```

## Consequences

- Open: field-name shorthand (e.g. `User(id:, name:)` when local names match
  field names), default field values, and whether fields may be given in any
  order (the canonical formatter could reorder them to declaration order).
- `none` in the example is illustrative. The spelling of the absent optional
  value is still to be decided.

## Rejected alternatives

- **No separator:** ambiguous.
- **Braces:** inconsistent with the language's visual style.
