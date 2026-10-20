# 0616. Native `any Trait`: a tagged box and a generated dispatcher

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.2](../spec/23-native.md), trait.any.1, trait.any.3, trait.self.1, own.val.1
- **Builds on:** [ADR 0536](0536-native-generics-by-monomorphisation.md) (impl blocks and defaults as methods of the type), [ADR 0392](0392-native-structs-and-methods.md) (heap header), [ADR 0079](0079-any-dynamic-dispatch.md)

## Context

`any Trait` values (`List[any Shape]`) need dynamic dispatch. The native
backend has no function values yet (lane 174) and no data relocations for
tables of code addresses.

## Decision

0. **`Self`.** In an `impl` method or an inherited default, `Self` in a
   type names the implementing struct (trait.self.1), so `fn
   Self.same(self, o Self)` lowers as a method of `S` taking an `S`.
1. **Representation.** An `any T` value is a pointer to a 16-byte heap box
   (kind 11): slot 0 holds a tag, the value's type's index among the
   structs implementing `T` in the program (in declaration order), slot 1
   the value.
2. **Conversion.** Where `any T` is expected (an argument, a field, a list
   element, a result), a struct value is copied into a new box
   (trait.any.3, own.val.1).
3. **Dispatch.** `x.m(args)` on an `any T` calls `any.T.m(x, args)`, a MIR
   function generated once per trait method used. It loads the tag,
   compares it with each implementation's index in turn, and calls `S.m` on
   the box's value. A `mut self` method changes the value in the box.
4. **Copies.** `any T` needs copying as a struct does. The generated
   `copy.any T` makes a new box with the same tag and a copy of the value
   for its type, so a copied list of shapes is independent.

## Consequences

- `examples/shapes.ail`'s `Drawing` of circles and rectangles lowers; what
  still blocks that file is generics, optionals and printing.
- A dynamic call costs one load, up to N-1 compares for N implementations,
  and a direct call, which can be inlined into the dispatcher.
- Printing an `any` value and impls for non-struct types are reported.
- When function values land, the dispatcher can become a table lookup
  without changing the box.

## Rejected alternatives

- **A vtable of function pointers per (type, trait).** One indirect call
  regardless of N, but needs code addresses in data and an indirect call
  op that lane 174 is still adding. It's the planned follow-up for traits
  with many implementations.
- **Fat values (tag and pointer in two registers).** Avoids the box, but
  every list, field and parameter of `any T` would need 16-byte slots, which
  the 8-byte value model doesn't have.
- **Monomorphise `any T` away.** Impossible for heterogeneous lists, the
  main use.
