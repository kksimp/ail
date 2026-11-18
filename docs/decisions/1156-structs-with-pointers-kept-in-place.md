# 1156. A list keeps structs with texts and lists in place

- **Status:** Accepted
- **Date:** 2026-11-18
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked on 2026-11-17 to keep closing the speed gaps to C without syntax changes
- **Topic:** 25 — Optimization strategy
- **Spec rules:** [native.flat.2](../spec/23-native.md)
- **Builds on:** [ADR 1149](1149-structs-kept-in-place-in-lists.md) (structs of numbers in place; its option D), [ADR 0562](0562-native-heap-size-classes-and-header.md) and [ADR 0564](0564-collection-trigger-and-sweep.md) (the heap header and the collector)

## Context

ADR 1149 keeps a struct in a list's own block only when every field is a
number. The structs real programs put in lists nearly always hold a text or
a list as well (a token with its text, a record with a name, a node with its
children), and each of those is still one heap object per element behind a
pointer.

Nothing in the lowering of ADR 1149 depends on what a field is: an element's
address has a struct object's shape, and a pointer field sits in its 8-byte
slot as it does in an object. Two things do depend on it:

1. **The collector.** A list header's scan reads `len` words of the element
   block (`gc_mark_list_data`). With `w` words per element that covers only
   the first `len / w` elements, so pointers in the rest would be missed and
   what they point at freed.
2. **Copies.** Copying the words of an element is a copy of the struct only
   while no field is itself copied by a copy (a list, a map, a set, a tuple,
   a struct; `needs_copy`). ADR 1149's `push`, `xs[i] = s` and list copy
   copy words.

## Options considered

### Option A — a block kind that carries the element size (chosen)

The element block of such a list has header kind `32 + w` (`w` words per
element, 1 to 31; `KIND_FDATA`). The list header's scan reads `len * w`
words of a block of that kind. A block reached another way (an element's
address on a stack, which the collector takes for the block) is queued and
scanned whole, as every other object is.

- Pro: three lines in the collector and none on its hot path for other
  kinds; no header bit is taken (`AIL_GC_VERIFY` checks bits 8 to 15 are
  clear), and the census and leak tables already index 64 kinds.
- Pro: the scan stops at the length, as for a list of pointers, so unused
  room is not read.
- Con: 31 kinds are taken from the 255.

### Option B — the element size in header bits 9 to 15

- Con: the header verifier treats those bits as corruption; every reader of
  a header would have to mask them.

### Option C — one kind, the whole block scanned

- Pro: no size needed.
- Con: unused room holds stale words (blocks are not zeroed), which keep
  dead objects alive; zeroing the room costs a pass on every growth.

### Option D — a pointer map per struct, for an exact scan

- Con: the collector is conservative everywhere else (objects are scanned
  word by word), so an exact scan here buys nothing until that changes.

## Decision

1. A struct is kept in place when it has one to eight fields, each a number
   (ADR 1149) or a `str`, `bytes`, list, map, set, optional, tuple, enum or
   struct, none of a unique type. Other field types (`ref`, pointers,
   atomics, channels, `any`) keep the struct boxed.
2. When some field is not a number the element block has kind `32 + fields`.
   `rt.flat_push_refs` makes and grows such blocks, and gives the kind in
   place to the block an empty list came with; `rt.flat_gather` does the same
   for a list built on the pool (conc.par.16); `rt.flat_copy_refs` copies
   one; `rt.flat_new_refs` makes an empty one with room.
3. The collector's list scan reads `len * (kind - 32)` words of such a block.
   The leak report's scan does the same.
4. Copies are those of a struct object. `xs.push(s)` and `xs[i] = s` copy
   `s` first when it is read from a place and some field is copied by a copy
   (the test a list of objects uses, `copy_if_place`); a copy of the list
   whose elements have such a field is a generated function that copies each
   element field by field; otherwise the words are copied.
5. `AIL_OPT_OFF=flatref` keeps the rule of ADR 1149 only; `AIL_OPT_OFF=flat`
   still boxes everything.
6. Which uses box a struct is unchanged (ADR 1149 decision 4).

## Rationale

AI understanding and readability are untouched: results, copies, order and
traps are those of the boxed list, and no syntax changes. The choice is on
speed: one allocation and one pointer hop fewer per element for the lists
programs actually have. Option A is the smallest change to the collector
that is exact about the list's length.

## Consequences

- More programs take a second round of lowering: a struct with a text that
  is also sorted, printed as a list or compared is found on the first round
  and boxed on the second. The cost shows in compile time, not in the
  program.
- The compiler's own lists of structs become in-place where its uses allow,
  so the self-host fixed point exercises the layout.
- An element with a list in it that is pushed from a variable still in use
  is copied to a temporary struct first, then its words are copied in.
- `tests/native/run/flat_refs.ail` checks growth under collection, the pool
  builder, and every copy against the value it was taken from.
- A struct of more than eight fields, or one given to a list method other
  than those of native.flat.1, is still boxed.

## Rejected alternatives

- **B (header bits):** breaks the header verifier's invariant for a size
  that fits in the kind.
- **C (scan the whole block):** keeps garbage alive through stale words, or
  pays to zero unused room.
- **D (exact pointer maps):** no gain while objects are scanned
  conservatively.
