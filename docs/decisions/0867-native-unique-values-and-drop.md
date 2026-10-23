# 0867. Native unique values and Drop: ownership flags on the defer machinery

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 7 — Ownership/lifetime semantics; 23 — Compiler architecture

## Context

`unique` structs and enums, `impl Drop`, `move` parameters and unique lists and
maps (own.uniq.1-10, own.move.1-6) only ran in the interpreter. Native builds
rejected `unique struct`, which blocked the ownership programs of the
coverage sweep.

## Options

### Option A — a hidden `live` flag per owned local, releases on the defer lists
Each unique local gets an SSA flag: set when bound, cleared when the value moves
away. A marker statement in the block's `defer` list releases it where the block
ends, at `return`, a failing `?`, `break` and `continue`, so the order (last
bound first) and every exit path reuse own.defer.1-3. A generated `drop.T`
runs `Drop.drop`, then the unique fields last first; options, lists and maps
release what they hold.
- **Pros:** one mechanism for defers and drops; the flag folds away on straight-line code; no change to value layout.
- **Cons:** one-line hooks where values are bound, moved, assigned or discarded.

### Option B — static move analysis only
Decide at compile time which locals still own their value at each exit.
- **Pros:** no flags.
- **Cons:** a conditional move needs a flag anyway; a second analysis next to the checker's.

### Option C — reference counts on unique objects
- **Pros:** no per-local state.
- **Cons:** a runtime cost on every value, and release order is not the declared order.

## Decision

Option A. Unique types are ordinary struct and enum objects (never copied: a
copy site moves instead). A moved-from local's flag is cleared at `move` arguments,
`move self` receivers, lets, assignments, returns and tail expressions. Assigning
a place (local, `xs[i]`, `m[k]`, `x.f`) releases the value it held; an unused
call result is released at the end of its statement; `xs.clear()` releases its
elements. A trap runs the pending releases and defers (lower_gtd_unwind.ail).

## Consequences

- `unique` programs build natively; `ail run` and `ail build` print the same output.
- Not yet: unique values captured by closures, and `with`.

## Rejected alternatives

- **Option B:** needs flags for conditional moves anyway.
- **Option C:** runtime cost on all values and the wrong release order.
