# 0865. Native maps and sets take struct, tuple, enum and list keys, hashed by the key's shape

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 4 — Types; native backend
- **Spec rules:** `type.map.*`, `type.coll.*`
- **Builds:** [ADR 0568](0568-native-map-and-set-layout.md)

## Context

Native tables took `int`, `bool`, `char` and `str` keys only. About twenty
programs in `bootstrap/tests/cases` (`maps_*`, `tuples_basic`) use struct,
tuple, enum or list keys, so they could not build natively.

## Options considered

### Option A — the table is told the key's shape (chosen)
The key kind of a table is a small integer, or, for a composite key, a string
object that describes the layout (`i` word, `f` float, `s` string, `L` list,
`S` struct or tuple, `E` enum with `V` variants). The runtime hashes and
compares keys by walking that shape. Keys are copied when stored.
- **Pros:** no new lowered functions, one runtime walker, the lookup fast path
  for `int` and `str` keys is unchanged (one extra comparison on the kind).
- **Cons:** a walk per hash of a composite key; recursive types are refused.

### Option B — lower a hash and an equality function per key type
- **Pros:** faster per key.
- **Cons:** the runtime would need function pointers into lowered code; much
  more compiler code for a rare case.

### Option C — store the key as its text
- **Pros:** no runtime change.
- **Cons:** iteration must recover the key; every table operation changes.

## Decision

Option A. A type is a key when its fields are, recursively, ints, chars, bools,
floats, strings, bytes, lists, tuples, structs or enums without a cycle.
`-0.0` and `0.0` are one key. A missing key traps with the key's text.

## Consequences

- `Map[P, V]`, `Set[(int, int)]`, `Map[List[int], V]` and enum keys build natively.
- Not yet: optional, map, set or recursive-type keys; tables whose type is
  settled by a later use (`{}` typed by first assignment) with a composite key;
  the trap trace does not show map arguments (a general native gap).

## Rejected alternatives

- **Option B:** needs calls into lowered code from the runtime.
- **Option C:** changes every table operation.
