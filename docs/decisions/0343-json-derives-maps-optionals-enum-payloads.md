# 0343. JSON derives for maps, optionals and enum payloads, with error paths

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 17 — Metaprogramming; 22 — Standard library (`std.json`)
- **Spec rules:** [json.derive.1–2](../spec/std-json.md) (amended), [json.derive.3](../spec/std-json.md) (new), [json.query.4](../spec/std-json.md) (amended)
- **Amends:** [ADR 0308](0308-tojson-fromjson-derives.md) points 1–2

## Context

ADR 0308's derives stop with `ct.derive.4` at maps, optionals other than
scalars, generic types and enum variants with fields, which agent configs
and tool messages use constantly. Their failures also said only `expected a
string, found number`, not where. **AI understanding decides**: an agent
fixing a rejected document needs the path; token counts of user programs
don't change (`impl FromJson for T` stays one line), and the generated code
is what `ail api --expanded` shows.

## Options considered

### Variant tagging
- **A — `{"kind": "Name", fields...}`, unit variants as `"Name"` (chosen):**
  flat, the shape most agent and tool APIs use (internally tagged); a reader
  can switch on one member. Cost: a variant field named `kind` clashes, so
  it is a `ct.derive.4` error.
- **B — `{"Name": {fields}}` (ADR 0308's `ToJson`):** no clash, but one more
  nesting level and a key that is data, which JSON schemas handle badly.
- **C — `{"kind": "Name", "value": {...}}` (adjacent):** no clash; longer.

### Error paths
- **A — the path in the message, built on the way out (chosen):** each
  generated read wraps its failure with `JsonError.under(part)`, so the
  success path costs nothing; messages read `missing items[2].kind` (as
  ADR 0308's `missing id` did) or `items[2].kind: expected a string, found
  number`. `JsonError` keeps its two fields.
- **B — a `path` field on `JsonError`:** structured, but every
  `JsonError(at, message)` changes, and the message still needs the text.
- **C — pass the path down into each `from_json`:** builds a string per
  member read even when nothing fails.

### Generic types
- **A — `ToJson` through bounds; `FromJson` a clear `ct.derive.4` until
  OQ-039 (chosen):** `impl ToJson for Pair[A ToJson, B ToJson]` already
  checks; `Pair.from_json` would have to call `A.from_json`, a trait
  function without `self` (OQ-039, a serious design).
- **B — derive `FromJson` per instantiation (`impl FromJson for Pair[int,
  str]`):** needs impls of concrete instantiations, which the checker
  doesn't have, and one name per type.

## Decision

1. `ToJson` writes a variant with fields as `{"kind": "Name", fields...}`;
   `Map[str, T]`, `List[T]` and `T?` fields convert element by element when
   `T` has no `to_json`.
2. `FromJson` reads `List[T]`, `Map[str, T]` and `T?` (missing or `null` is
   `none`) nested to any depth, and enum variants with fields from objects
   tagged by `kind`; a unit variant reads from `"Name"` or `{"kind": "Name"}`.
3. Failures name their path (`json.derive.3`). New prelude helpers:
   `Json.need_obj`, `Json.non_null`, `Json.opt`, `Json.variant_name`,
   `JsonError.under`.
4. `FromJson` for a generic type, a map with non-`str` keys, a set, and a
   variant field `kind` are `ct.derive.4` errors.

## Consequences

- An optional scalar field of the wrong type is now an error, not `none`.
- `examples/config.ail` reads a service config with a map, an optional enum
  with payloads, and optional floats in one call.
- Token counts of `tasks.ail` and `merge.ail` are unchanged (their source
  is unchanged); check time grows with the generated code (see the PR).

## Rejected alternatives

- **`{"Name": {fields}}` (B) and adjacent tagging (C):** see above.
- **A `path` field (B) or paths passed down (C):** see above.
- **Per-instantiation generic `FromJson`:** needs concrete-instantiation
  impls; wait for OQ-039 instead.
