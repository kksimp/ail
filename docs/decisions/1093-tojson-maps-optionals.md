# 1093. `ToJson` on `Map[K, V]` and `T?` without an `impl` block

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), from the designer's approval of every AI-first review item for 0.1.0 (2026-11-12)
- **Topic:** 17 — Standard library (JSON)
- **Spec rules:** [json.derive.4](../spec/std-json.md)
- **Amends:** [ADR 0343](0343-json-derives-maps-optionals-enum-payloads.md) (maps and optionals only as derived fields); finishes the item [ADR 1087](1087-std-papercuts.md) deferred

## Context

The language card says the `ToJson` derive covers lists, `Map[str, T]` and `T?`.
That held only for fields of a derived struct, which the generator writes out
element by element. At top level, `print_json(m)` on a `Map[str, Stats]` and
`x.to_json()` on an `int?` were `gen.bound.3` errors: only `List[T ToJson]`
has an `impl`, through a builtin generic owner wired into the checker and the
lowering. A log analyzer an agent wrote in the AI-first review failed on it.

## Decision

- `Map[K, V]` is `ToJson` when `V` is, and `T?` when `T` is, structurally, the
  way `T?` is `Eq` when `T` is (`trait.auto.1`). No `impl` block, no new syntax.
- A map gives an object of its entries in insertion order, each member named
  by its key's text form (`"{k}"`); `none` gives `null`, a present value its
  own JSON.
- The checker's `implements` has the rule; the help for a missing `ToJson`
  names the innermost value type and says that a bare `impl ToJson for U`
  derives it.
- The lowering turns `.to_json()` on a map or optional into an instance of the
  prelude's generic `json_of_map[K Hash, V ToJson]` or `json_of_opt[T ToJson]`, keyed
  `Map[str, int].to_json` like a `List[T]` method (`compiler/mir/lower_json_builtin.ail`).

## Rejected alternatives

- `impl ToJson for Map[K, V ToJson]` and an `impl` for `T?` in the prelude: the
  builtin generic owner takes one parameter today, so this needs two-parameter
  owners in the checker (`collect_impls`, `con_of`, `args_of`) and the lowering
  (`BUILTIN_OWNERS`, `builtin_method`, `assoc_owner`), plus a new target syntax
  for `T?` that nothing else uses. More surface for the same behaviour.
- Only `str` keys: JSON members are strings, but agents write `Map[int, V]`
  histograms and expect `{"404": ...}` (Python's `json.dumps` does the same).
  `FromJson` keeps `str` keys only (`ct.derive.4`), since reading back an
  `int` key from its text would need a rule of its own.
- A runtime `Json` conversion of any value (reflection): loses the
  compile-time `gen.bound.3` check that names the type without `ToJson`.
