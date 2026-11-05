# 0937. `Grid[T]`: a flat 2-D list with `g[i, j]`

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 04 — Types; performance
- **Spec rules:** [type.grid.1-6](../spec/04-types.md)
- **Builds on:** [ADR 0029](0029-token-efficiency-benchmarks.md) (measure syntax), [ADR 0808](0808-native-prelude-json-lowered-on-demand.md) (prelude modules), speed-edges proposal P11

## Context

2-D data (DP tables, boards, images, matrices) is written as
`List[List[T]]`: every row is its own object, building it takes a nested
comprehension, and `g[i][j]` is three dependent loads. `std.tensor` is flat
but holds numbers only and has no element assignment.

## Decision

1. **A prelude type,** `struct Grid[T]` with `rows`, `cols` and `cells` (a
   `List[T]` row by row), in a `grid` prelude module read only when a program
   names `Grid` and doesn't declare it. `Grid.fill(rows, cols, v)`,
   `Grid.from(xs)`, `g.row(i)`, `g.to_list()` are AIL code in
   `tools/native/prelude.ail`. A grid is a struct, so copies, `==`, printing,
   fields of structs and `mut` parameters need nothing new.
2. **`g[i, j]`.** The parser reads `x[a, b]` as `x[(a, b)]` (an `Index` of a
   tuple literal; no new syntax node). The checker types it as the element on
   a `Grid` (type.grid.3) and as a tuple key elsewhere (type.grid.6). Native
   lowering (`lower_grid.ail`) checks `i` against `rows` and `j` against
   `cols` with the `bounds` instruction (type.coll.4), so `i * cols + j`
   can't overflow, then reads `g.cells` at that index.
   Reads, `=`, `op=` and the assignment-order rule (ADR 0131) are the list
   element's.
3. **Size fields are read-only** (type.grid.2): assigning `rows`, `cols` or
   `cells` is a compile-time error.

## Measurements

Tokens (`tools/tokbench`, five-tokenizer mean), the same function written
both ways (files in the PR description):

| Program | `List[List[T]]` | `Grid[T]` | Change |
|---|---:|---:|---:|
| LCS table | 140.4 | 131.8 | -6.1% |
| matmul (ikj) | 111.8 | 93.0 | -16.8% |
| life step | 153.2 | 134.2 | -12.4% |

The language card grows by one sentence (7078 to 7147 tokens of 8000); the
quick reference pays for its line with trims (ADR 0817).

Speed (ail-linux, min of 7; same algorithm both ways, outputs equal):

| Program | `List[List[int]]` | `Grid[int]` | C (gcc -O2) | Python |
|---|---:|---:|---:|---:|
| matmul 600 (ijk) | 0.40 s | 0.80 s | 0.24 s | 17.97 s |
| LCS 3000 | 0.09 s | 0.20 s | | |

A grid is 2x slower than nested lists today: bounds-check elimination
(ADR 0577) removes the nested lists' checks in counted loops, and nothing yet
removes the two `bounds` checks of `g[i, j]` (matmul: 34 instructions per
inner iteration against 12). Hoisting the grid's slot loads out of the loop
was tried and measured no change (7.39G to 7.40G instructions) at +6%
self-build time, so it was dropped. The win waits on versioning the loop
against `rows` and `cols` (speed-edges P2).

## Rejected alternatives

- **`g[i][j]` on a grid** (row views): needs a view type or a row copy per
  access; `g[i, j]` is fewer tokens and one access.
- **Methods only** (`g.at(i, j)`, `g.set(i, j, v)`): `g[i, j] += 1` becomes
  `g.set(i, j, g.at(i, j) + 1)`, about twice the tokens.
- **A compiler-built type like `List`** (own layout, runtime functions, both
  backends): more code in every pass for the same loads the struct gives;
  the optimiser's list passes (bounds hoisting, load forwarding) already apply
  to `cells`.
- **Extending `std.tensor`**: numbers only, needs `use std.tensor`, and its
  value is immutable.
- **`g.cells[g.flat(i, j)]` with a prelude `flat` method** (the first
  version): measured 3.1x the instructions of nested lists on matmul 600
  (8.27G against 2.64G on ail-linux), since the generic method call stayed a
  call; the checks are now emitted inline.

## Consequences

- Indexing a grid reached through a call (`make()[i, j]`) isn't in the native
  subset yet: the receiver is read twice, so it must be a variable or a field.
- `cells` can still be changed through methods (`g.cells.push(x)`); that
  breaks no memory rule (every index is still checked), only the shape.
- Proven-parallel loops (ADR 0911) don't take `g[i, j]` writes yet; the
  data-range work (P2, P3) can extend them.
- Until P2, a grid is the shorter way to write 2-D code, not the faster one.
