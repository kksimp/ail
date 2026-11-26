# 1201. `for i, x in xs.enumerate()` walks the list with a count; a join bound by the statement before is a map key lookup too

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's speed mandate of 2026-11-17 (runtime speed only, no syntax change) and the Mac fast track of 2026-11-25
- **Topic:** Native lowering
- **Spec rules:** native.map.1 (the sentence of ADR 1197, widened); type.tuple.9 and expr.loop.3 unchanged
- **Builds on:** [ADR 1197](1197-bytes-loops-in-place-and-joined-map-keys.md)

## Context

Two forms next to those of ADR 1197 still did work the program had no use
for.

- `for i, x in xs.enumerate()` lowered `xs.enumerate()` as a value: a new
  list of `(index, element)` pairs, one tuple object an element, which the
  loop then took apart. This is the form the checker's own fix writes for
  a two-name loop over a list (type.tuple.5), so it is common. Twenty
  passes over a list of two million `int` took 0.354 s and 374 MB.
- ADR 1197 looked a joined key up only when the join was written in the
  key (`counts[cur.join("")]`). `w := cur.join("")`, then `counts[w]`, still
  made the string for every occurrence.

`for b in bs` in a comprehension was already in place after ADR 1197 (a
comprehension lowers as a `for`), and `bytes` has no `enumerate`.

## Options considered

### Enumerate loops

- **A: `for x in xs` with a hidden count (chosen).** When the loop's
  iterable is `XS.enumerate()` on a list and its body starts by naming the
  two parts, the loop is rewritten before lowering as `for x in XS` with a
  hidden `mut` count that `i` reads and the first statement steps
  (`compiler/mir/lower_enumwalk.ail`). `for x in xs` already sees the list
  as it was when the loop started (it copies when the body may change the
  list), which is what the list of pairs gave. A `mut` or `_` element gets
  a hidden loop variable and its own binding.
- **B: keep the pairs in place (`List[(int, T)]` as in-place structs).**
  Still a list the size of the input, built to be read once.
- **C: an index loop `for i in 0..xs.len()` with `xs[i]`.** Evaluates `XS`
  again each turn and loses the snapshot.

### Joined key bound first

- **A: rewrite the join where it stands (chosen).** At a `str`-keyed
  lookup or store, when the last instruction of the block is the
  `rt.list_join_chars` call that made the key and its separator is the
  empty literal, that call becomes `rt.mapkey_chars(m, xs)` with the same
  result value. Nothing was emitted since the join, so the map was made
  before it and the list is unchanged. A variable bound to the join names
  the map's own equal key or the new string: equal text either way, and
  strings have no identity. This also replaces the syntactic test of
  ADR 1197.
- **B: follow the variable to a later lookup.** Needs proof that the list
  and the map are unchanged in between, and that every other use of the
  variable can take the map's key; the statement-before form is the one
  programs write.

## Decision

Both A. No syntax and no observable behaviour changes.
`AIL_OPT_OFF=enumwalk` keeps the pairs; `AIL_OPT_OFF=joinkey` keeps the join.
`xs.enumerate()` as a value, and over a `Seq`, a set or a deque, is as
before.

## Rationale

ADR 0076 order: nothing changes for the reader or the diagnostics, so speed
decides.

## Consequences

- Test `enum_walk`; `map_join_key` gains the bound form. Both, with
  `bytes_walk` and `for_copy_paths`, also pass with each fast path off.
- Mac (Apple Silicon, 2026-11-26, `bytejoin-mac-323cd390.txt`, branch
  against main 6316a803, best of 5, C twins with `cc -O2`):

  | Program | main | branch | C | main MB | branch MB |
  |---|---|---|---|---|---|
  | 20 passes of `for i, x in xs.enumerate()`, 2 M `int` | 0.354 s | 0.022 s | 0.009 s | 374 | 17 |
  | word count, join bound the statement before, 107 MB | 0.700 s | 0.527 s | 0.483 s | 244 | 116 |

  The byte hash and the word count of ADR 1197 are unchanged (0.668 s and
  0.527 s). The C twin of the first row is vectorized by the compiler; the
  AIL loop is not.
- Not covered: a joined key used more than one statement later.

## Rejected alternatives

- **In-place pairs:** still builds a list to read it once.
- **An index loop:** re-evaluates the list and loses the snapshot.
- **Following the variable:** more proof than the common form needs.
