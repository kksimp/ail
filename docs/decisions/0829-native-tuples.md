# 0829. Native tuples

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the layout was agreed with the native generics lane and accepted by the integrator)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** type.tuple.1-9, own.val.1, native.subset.1
- **Builds on:** [ADR 0791](0791-tuples.md) (the language), [ADR 0392](0392-native-structs-and-methods.md) (structs), [ADR 0818](0818-native-struct-and-enum-map-values.md) (copies), [ADR 0568](0568-native-map-and-set-layout.md) (table keys)

## Context

Tuples (ADR 0791) ran on the interpreter, the bytecode tier and the
self-hosted front end. The native backend reported them as outside the subset.
Generic instances, enum payloads and lists already carry any one-word value,
and struct objects already have the copy, equality and field-access code a
tuple needs.

## Decision

- **Layout.** A tuple `(A, B, ...)` is a pointer to a struct object (heap kind
  4) with one 8-byte slot per element, in order, with no header of its own.
  `t.k` is a load of slot `k`, as for a struct field. mir `Ty` gains
  `Tuple(elems)`, named by its source spelling, `(int, str)`; generic
  instances and the generated functions key on that name.
- **Values (own.val.1).** Reading a tuple from a place to bind, store, push or
  return it copies it, through a generated `copy.(A, B)` that copies elements
  which are lists, structs or tuples. `t.k = v` and `t.k op= v` change the
  object in place, which is safe because every place holds its own.
- **Elements.** Numbers, `char`, `str`, lists, structs, enums, optionals and
  tuples. Tables, tensors and function values are not elements: a copy would
  share state, and the type is reported as outside the subset.
- **Operations.** Literals (typed by the expected tuple), `t.k`, `a, b := t`,
  `for a, b in xs` and the same clause in comprehensions, tuple patterns in
  `match` (literals, names, `_`, nested tuples, enum variants), `==` and `!=`
  through a generated `eq.T`, `<` `<=` `>` `>=` through a generated `lt.T`
  (integers, booleans, characters, strings and tuples of those), `print` and
  interpolation of tuples of `int`, `float`, `bool`, `str` and tuples, and
  `xs.enumerate()` and `zip(a, b)` over native lists.
- **Out of the subset, each a `native.subset.1` report and never a wrong
  program:** tuples as map keys and set elements (native tables key `int`,
  `bool`, `str` and `char`: ADR 0568), `sorted`, `contains`, `index_of`,
  `min`, `max` and `counts` on a list of tuples, `<` on a tuple with a float
  element (NaN has no order to keep), and the text of a tuple that holds a
  list, `char` or enum.
- **Code.** A new `compiler/mir/lower_tuple.ail`; each existing file gets a
  dispatch line or an arm (ADR 0823's rule).

## Consequences

- Programs with pairs and triples build natively; the examples that stopped at
  "a tuple" move on to their next blocker.
- A tuple costs one allocation, like a struct; `enumerate` and `zip` allocate
  one per element.

## Rejected alternatives

- **Elements inline in a list slot.** A `List[(int, int)]` could store 16 bytes
  per entry, but lists have 8-byte slots (native-abi.md §2) and every generic
  path assumes one word per value. The pointer costs an allocation per pair
  and keeps all of those paths unchanged.
- **Tuples as anonymous generated structs.** It would need a name table and a
  registration step per type for no gain: the type's spelling is already a
  unique key.
- **Hashable tuples in tables now.** It needs a key kind in the table runtime
  and the hash and equality calls for it. It waits for a table change of its
  own.
