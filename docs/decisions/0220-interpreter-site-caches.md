# 0220. Cache what a call site names, by the address of its expression

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Interpreter (Stage 2), [ADR 0032](0032-keep-rust-interpreter-as-oracle.md)
- **Spec rules:** none (no behaviour changes)

## Context

Every call looked up its callee three ways (struct, variant by position,
function by name), every method call hashed the checker's call table, the
method name and the type name, and every struct or variant construction
re-derived which field each argument sets (a vector of pairs, built from
string compares). In `interp.ail` this was about 6% of the run. What a site
names never changes while the program runs.

## Options considered

### Option A — per-site caches keyed by the address of the expression (chosen)
- **Pros:** one integer hash replaces several string hashes and compares; no
  change to the AST, so no other lane's pattern matches break; the program is
  immutable for the interpreter's lifetime (`&'a Program`), so addresses are
  stable and unique.
- **Cons:** the key is an address, not a name; correctness rests on the AST
  never being freed or rebuilt while the interpreter runs. The interpreter
  never builds expressions of its own, and doc tests are appended to the source
  before parsing.

### Option B — resolve the target in the AST
- **Pros:** no maps.
- **Cons:** adds a cell to `Expr::Name` and `Expr::Call`, changing every
  pattern match in every module while ten lanes edit them.

### Option C — resolve in the checker's `Resolved` by position
- **Pros:** the checker already keys several tables by position.
- **Cons:** still a hash of two integers per call; couples the interpreter to
  more checker state.

## Decision

Cache in the interpreter, keyed by `callee as *const Expr` (calls), the
receiver expression (method calls: the checker's trait dispatch, whether the
method may change its receiver, and the last user method found for the
receiver's type), and the argument slice (the field each argument sets).
String literals are cached the same way, by the address of their text.

## Rationale

ADR 0076 criterion 4 (speed); criterion 2 holds because each cache is filled
by the code it replaces, and the method cache re-checks the receiver's type on
every use.

## Consequences

- A change that makes the interpreter evaluate expressions it builds itself,
  or run more than one program in one `Interp`, must not use these caches for
  them.

## Rejected alternatives

- **Key by source position:** positions are shared between a call's callee and
  the call, and a hash of two integers costs more than one.
