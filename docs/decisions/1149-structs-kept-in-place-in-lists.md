# 1149. A list keeps structs of numbers in place

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 for lanes on the speed gaps still open
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.flat.1](../spec/23-native.md)
- **Builds on:** [ADR 0392](0392-native-structs-and-methods.md) (struct objects), [ADR 0562](0562-native-heap-size-classes-and-header.md) (the collector takes an address inside a block), [ADR 0060](0060-mutable-value-semantics.md) (values are independent)

## Context

A `List[S]` holds one pointer per element, each to a struct object of its
own. For a struct of four `float` fields that is 8 bytes of pointer and a
48-byte block (header, four slots, rounded to 16) against the 32 bytes a C
array of the struct takes. A loop over a million such elements is bound by
memory: the `aos` micro ran at 1.72 times its C twin on ail-linux
(2026-11-17), and the same data kept as four `List[float]` ran level with C.
Nothing short of keeping the fields in the list's own block closes that.

Lowering leans on an element being a pointer to its fields: `xs[i].f = v`,
`for mut`, a `mut` argument and a method on `xs[i]` all change the element
through that pointer, and about fifty places read or write list elements.

## Options considered

### Option A — the element's value is the address of its fields in the block (chosen)

Element `i` of an in-place list is the bytes at `data + i * 8 * fields`, and
`xs[i]` evaluates to that address. It has a struct object's shape, so every
field read and write, `mut` parameter and `for mut` lowers unchanged, with
the instructions the optimizer already tracks as struct slots. Only what
makes, grows, copies or indexes the list knows the layout.

- Pro: the common uses need no new code and no new MIR operation.
- Pro: the collector already keeps a block alive for an address inside it.
- Con: the address names a slot, not an object: it must not outlive a change
  to the list. Lowering copies a struct read from a place before keeping it
  and reads in place only when nothing changes the list meanwhile, so this
  holds today; a new way to keep an element without a copy must check it.

### Option B — element fields read and written as list elements at `i * fields + k`

- Pro: no address of an element exists.
- Con: `xs[i]` as a value needs a new object each time it is passed, looped
  over or used as a receiver, so the programs that pass elements to
  functions get slower, or their structs stay boxed.
- Con: every place that changes an element through its pointer needs a
  second form.

### Option C — rewrite a local `List[S]` into one list per field

- Pro: no layout change.
- Con: a list passed to a function, kept in a struct or returned can't be
  rewritten, and those are the lists real programs have.

### Option D — in-place storage for every struct, pointer fields included

- Con: the collector scans a list's block by its length in words and would
  miss pointers in later elements; it needs a new block kind. Left for a
  later decision, with measurements of structs that hold a `str`. Decided in
  [ADR 1156](1156-structs-with-pointers-kept-in-place.md).

## Decision

1. A struct is kept in place in lists when it has one to eight fields, each
   an `int`, `float`, `bool` or `char`, and is not a unique type. The
   element block then holds no pointers (kind 11).
2. `xs[i]` is a bounds check and the address `data + i * 8 * fields`;
   `xs[i] = s` and `xs.push(s)` copy the fields in; a `for` variable is the
   element's address; a copy of the list copies the block (`rt.flat_copy`).
   `rt.flat_push` grows the block and returns the new element's address.
3. A list's room is read from its block's header, not from `cap`: a list
   made empty before its element type was known has a block of 8-byte
   elements, and is still a valid empty in-place list.
4. The layout is chosen per struct for the whole program. Each function is
   read when its lowering ends, with the types its values ended with: an
   instruction that reads or writes the elements of a `List[S]` another way
   (`get`, `set`), a runtime call given or returning a value that holds a
   `List[S]`, a primitive reading at one, or a copy that gives one another
   type, boxes `S`. A round of lowering that boxed a struct stops before the
   runtime is lowered and runs again with the set; after three such rounds
   every list is boxed. A program with no such struct is lowered once.
5. `AIL_OPT_OFF=flat` keeps every list boxed.
6. Results, order, copies and traps are those of the boxed list.

## Consequences

- A program whose every use of a `List[S]` is an index, a field, `push`,
  `len`, `for`, a copy or a call of its own functions gets the in-place
  layout; one use of `pop`, `sort_by`, a list literal, `==` or text keeps
  `S` boxed everywhere. Each of those can be given an in-place form later
  without changing the rule.
- A program that boxes a struct of numbers is lowered twice.
- Runtime functions that receive a list must not be given an in-place one
  unless `flat_knows` names them.
- `tests/native/run/flat_lists.ail` checks reads, writes, growth and copies
  against lists of the fields kept beside the list.
