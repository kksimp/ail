# 0392. Structs and methods in the native subset

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), own.val.1, trait.auto.2
- **Builds on:** [ADR 0384](0384-mir-shape-and-native-subset.md) (MIR), [ADR 0390](0390-native-subset-for-the-drag-race.md), [ADR 0060](0060-mutable-value-semantics.md) (values)

## Context

The designer wants AIL off the interpreter as fast as possible: the native
backend must compile the AIL compiler. A survey of `compiler/` found about
600 of its 937 native-subset problems in methods and struct types. The
heap layout must also be fixed now, so fifteen parallel lanes (maps,
closures, optionals, the collector) can build objects that agree.

## Decision

1. **Heap header.** Every heap object gets an 8-byte header word before
   the address the program holds: kind (bits 0–7), mark (bit 8), size
   (bits 16–63). Pointers keep pointing at the first field, so the string
   and list layouts and every offset in the code generators stay as they
   are. `rt.obj_new(size, kind)` allocates; the kind table is in
   native-abi.md §2.
2. **Struct objects:** one 8-byte slot per field in declaration order,
   read and written with two new MIR operations, `load` and `store`, at a
   constant byte offset (one instruction on each target).
3. **Value semantics** (own.val.1) by copying at the boundary, as lists
   already do: a struct or list read from a place (variable, field,
   element) is copied when bound, stored, pushed, or returned when it
   belongs to someone else (a parameter, `for` variable, field or
   element). Copies are deep through lists and structs; a type that holds a
   struct gets a generated `copy.T` function. Fields then change in place,
   which is safe because no two places share an object.
4. **Methods** are functions named `T.m` with `self` first; `mut self` and
   `mut` parameters of struct type pass the object, so changes reach the
   caller. Inside a method a bare field name is `self`'s field and a bare
   call can name a method of `T`; `T.f(...)` calls a function of `T`
   without `self`.
5. **Printing** lists of lists and of strings follows the interpreter
   (strings quoted and escaped, trait.auto.2). Printing a struct stays
   outside the subset.
6. After the first problem in a function its MIR isn't verified or kept,
   so a construct outside the subset is reported once, not also as an
   internal error.

## Consequences

- The struct tests build and match `ail run` on x86-64 and ARM64 (qemu).
- A copy costs an allocation per object and list it holds; the interpreter
  pays the same on writes. Copy elision for values that are never used
  again is a later optimisation.
- Generic structs, invariants and error structs stay outside the subset
  (generics are next, by monomorphisation).

## Rejected alternatives

- **Structs in registers or on the stack.** Faster for small structs, but
  the compiler's structs hold lists and are stored in lists; one
  representation keeps lowering, copies and the collector simple. Scalar
  replacement can come later as an optimisation.
- **Copy-on-write with reference counts** (as the interpreter's `Rc`
  values). Every read of a field would pay a count; copying at the
  boundary costs only where values are duplicated, and the compiler's code
  rarely duplicates them.
- **A header at offset 0.** Every string and list offset in two code
  generators and the runtime would shift.
