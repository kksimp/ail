# 0774. `load_json[T](path)` and `print_json(v)`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library (`std.json`)
- **Spec rules:** json.load.1, json.load.2 ([std-json](../spec/std-json.md))
- **Builds on:** [ADR 0748](0748-list-from-json-as-an-associated-call.md), ADR 0631

## Context

Twin t11 (read a JSON file of users, keep adults, print the result) spends 89.6
tokens against Python's 57.6. Most of the extra is the chain an agent writes
every time it reads typed JSON from a file and prints typed JSON:
`T.from_json(parse_json(fs.read_text(p)?)?)?` and `v.to_json().to_text()`.

## Decision

Two generic functions in the `std.json` prelude, read when the program names
`FromJson` or `ToJson` (like the derives): `load_json[T FromJson](path str) T !
JsonError` and `print_json[T ToJson](v T)`. A missing or unreadable file fails
with a `JsonError` whose message starts with the path and the I/O kind, so one
error type covers reading, parsing and shape. Names follow `parse_json`, so no
module namespace is needed (`json.` does not exist, ADR 0161's TODO).

Same tokens as the dotted form `json.load[T]` the first sketch used (both 76.4).

## Token measurement (`tools/tokbench`, five-tokenizer mean)

| Twin t11 | tokens |
|---|---|
| today: `List[U].from_json(parse_json(fs.read_text(p)?)?)?` and `.to_json().to_text()` | 89.6 |
| `load_json[List[U]](p)?` and `print_json(v)` | **76.4** (-13.2) |
| same, and no `impl ToJson/FromJson for U` lines (implicit derive, not built) | about 64 |

Python 57.6. Language card +15.4 tokens (6,949.2 to 6,964.6; budget 8,000). The 25 twins would drop 13.2 tokens (1,128.6 to about 1,115.4,
+12.6% against Python). Dropping the two `impl` lines would be another 12.4 (see the rejected
alternatives).

## Rejected alternatives

- **`json.load[T]`:** same tokens, needs a `json` namespace that does not exist.
- **Reading the file inside `T.from_json`:** conflates decoding and I/O.
- **A new error type for I/O:** one more name; `JsonError.message` carries it.
- **Implicit derive of `ToJson`/`FromJson`** (no `impl` lines): -12.4 more tokens
  on t11, 1.1% of the 25-twin total, a tie under the 2% rule; and the explicit
  empty `impl` keeps visible that a type crosses a JSON boundary. Revisited if
  the whole-task reruns show the `impl` lines costing more than 2%.
