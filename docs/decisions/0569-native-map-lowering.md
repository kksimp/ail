# 0569. Lowering `Map` and `Set` operations

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.map.1, type.map.3, type.map.5, type.map.6, type.map.7, type.map.8, type.coll.8, err.else.1, own.val.1, native.subset.1
- **Builds on:** [ADR 0568](0568-native-map-and-set-layout.md) (the table)

## Context

[ADR 0568](0568-native-map-and-set-layout.md) fixes the table. The lowering
(`compiler/mir/lower_map.ail`) must map the language's map and set forms to
runtime calls without a new MIR operation, and trap exactly as the
interpreter does.

## Decision

`Ty` gains `Map(key, val)` and `Set(elem)`. Every construct below is a
runtime call or a load of a table word (`len`, an entry's value):

| Source | Lowering |
|---|---|
| `Map[K, V]()`, `Set[T]()`, `{k: v, ...}`, `{x, ...}` | `rt.map_new(kind, vmode, vinfo)`, then `rt.map_set` / `rt.map_add` per entry |
| `m[k]` | `rt.map_at(m, k, head, tail)`; a missing key traps `type.map.3` with the interpreter's first line (`the map has no key K`, a `str` key quoted and escaped) and excerpt |
| `m[k] = v` | `rt.map_set` (replace in place or insert last); `k` is evaluated before `v` |
| `m[k] op= v` | `rt.map_find`; a missing key starts from `0` / `0.0` (type.map.7) |
| `m.get(k) else X`, as a binding or an operand | `rt.map_find`, a branch on 0, the entry's value or the `else` (a value, or a statement that leaves) |
| `m.contains(k)`, `s.contains(x)`, `s.add(x)`, `s.remove(x)` | `rt.map_has` / `map_add` / `map_remove`, returning `bool` |
| `m.remove(k)` | `rt.map_remove`, as a statement only (its value is an optional) |
| `m.len()`, `s.len()` | a load of the first word |
| `m.keys()`, `m.values()` | `rt.map_keys` / `map_values` (the `bool` forms build byte lists) |
| `for x in s` | a snapshot `rt.map_keys(s)` walked as a list (type.map.5) |
| `m.push(k, x)` on a `Map[K, List[T]]` | find, start an empty list when missing, push (type.map.8) |
| `print(m)`, `print(s)` | `rt.out_map` / `rt.out_set` for `int`, `str`, `bool` and `float` keys and values (type.map.4) |
| binding or pushing a table from a place | `rt.map_copy` (own.val.1) |

- The trap's head (`PATH:LINE:COL: trap[type.map.3]: `) and source excerpt
  are string constants built at the read, in the form the code generator
  builds for a checked operation; the runtime only prints them.
- A `float` value is stored and loaded through the `float_bits` and
  `bits_float` primitives; a `bool` is a word of 0 or 1.
- `mut` parameters of table type share the table, as lists do.
- Reported under native.subset.1: `m.get(k)` and `m.remove(k)` used as a
  value (optionals are lane 175's), `==` on tables, `sorted`, `union`,
  `intersect`, `difference`, `is_subset`, and keys that are not `int`,
  `bool` or `str`. `for k in m` and `m.items()` are errors in the language
  itself (type.map.5) and never reach the backend.

## Consequences

- No new MIR operation, no change to the code generators: a table works on
  x86-64 and ARM64 with the runtime calls alone.
- The lowering hooks into `lower.ail` at ten points, each a line or two
  that dispatches into `lower_map.ail`.

## Rejected alternatives

- **A MIR `mapget` / `mapset` operation.** Would let the optimiser see them
  (hoisting a repeated lookup), but needs both code generators and the
  verifier to change; the runtime calls can be inlined later.
- **Lower `get` to an optional value and `else` generically.** The right
  long-term form, and lane 175 owns it; this lowering recognises
  `m.get(k) else ...` directly so maps do not wait on it.
- **Trap from the runtime with a record per site.** A string constant per
  site is what the existing stubs do and needs no new data format.
