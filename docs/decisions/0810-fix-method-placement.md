# ADR 0810: `ail fix` moves methods out of `impl` blocks and structs

- Status: Accepted
- Decided by: AI (delegated, ADR 0076)
- Date: 2026-10-21

## Context

Two mistakes from other languages have one correct AIL spelling. `impl Point`
with indented `fn` lines (trait.impl.1), and `fn` lines inside a `struct`
(type.struct.2). The diagnostics named the rewrite, `fn Point.get(self) T`,
but did not apply it.

## Decision

- trait.impl.1 (`impl T` with no `for`): when no `trait T` is declared in the
  file, delete the `impl` line, move each line of the block left one indent
  step, and write `T.` before each method name.
- type.struct.2: when the methods are the last members of the struct, do the
  same for them, naming the struct.
- No fix when a member of the moved block is not a plain `fn` (an attribute, a
  field after a method), when a method already has a type name, or when `T` is
  a declared trait (the author probably left out `for Type`).
- Comment-only lines inside the block are not tokens and keep their indent;
  `ail fix` formats the file afterwards when it was formatted before.

## Rejected alternatives

- Fixing `impl Trait` by adding `for Type`: the type is a choice.
- Moving a method that sits between fields: the struct would be split.
