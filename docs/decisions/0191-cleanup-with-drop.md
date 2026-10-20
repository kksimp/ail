# 0191. Unique types clean up with the prelude trait `Drop`; where unique values may live

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 7 — Ownership; 21 — Standard library
- **Spec rules:** [own.uniq.5–7, own.defer.2–3](../spec/07-ownership.md#unique-types-and-cleanup)

## Context

ADR 0066 gives unique types a cleanup operation without a syntax, and says
unique values in shared structures need rules. The bootstrap copies values
freely in collections and generic code, so a unique value there would be
duplicated and released twice.

## Options considered

### How a cleanup is declared
- **A prelude trait, `impl Drop for File` with `fn drop(mut self)`
  (chosen):** the existing `impl` mechanism, visible in `ail api`, and the
  checker can verify the signature and that only unique types implement it.
  Familiar meaning. Con: about 4 more tokens per type than a method with a
  reserved name.
- **A method with a reserved name, `fn File.close(mut self)`:** fewest
  tokens, but a magic name is invisible in the type system and collides
  with ordinary `close` methods that report errors.
- **`fn File.close(move self)` called explicitly:** what the lane brief
  suggested, but ADR 0066 releases at scope end automatically; an explicit
  call would have to switch that off.

### Where unique values may live
- **Locals, `move` parameters, and fields of unique types (chosen):** a
  field of a unique type makes its owner unique; lists, maps, sets, `ref`
  cells and type arguments can't hold one. Optionals and results can, so
  `open(path) File ! IoError` works.
- **Everywhere, with copy checks in generic code:** needs a `Copy`-like
  bound on every generic parameter; a larger design (OQ-050).

### Release details
Chosen: `drop` runs first, then the unique fields, last first; assigning a
new value releases the old one; a call's unused unique result is released at
the end of its statement; `if x := e` owns `x` for the branch; deferred
statements and cleanups of one block run together in reverse order of their
`defer` and declaration. A deferred statement can't leave the block, declare,
or move, and what it uses must not be moved before the block ends.

## Decision

A unique type's cleanup is `impl Drop` with `fn drop(mut self)`, never
called directly; unique values live only in locals, `move` parameters,
optionals, results, and fields of unique types.

## Rationale

AI understanding decided (criterion 2): tokens differ only in a rare
construct (one `impl` per resource type), which ADR 0076 counts as a tie. A
trait is checkable and discoverable; a reserved name isn't. The placement
restriction is what keeps "released exactly once" (own.uniq.3) true in a
bootstrap that copies values.

## Consequences

- `Drop` joins the prelude traits.
- `List[File]` and generic helpers over files are rejected for now (OQ-050). Amended by [ADR 0318](0318-unique-values-in-lists-and-maps.md): lists and maps may now hold unique values.
- The interpreter releases values only when the program declares a unique
  type, and runs `defer` only when the program uses it; other programs pay
  one flag test per block.

## Rejected alternatives

- **Reserved method name:** invisible to the type system.
- **Explicit `move self` cleanup:** contradicts automatic release.
- **Unique values everywhere now:** needs a copyability bound first.
