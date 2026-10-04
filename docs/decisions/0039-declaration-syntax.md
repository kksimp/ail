# 0039. Declarations use `:=`, mutable declarations use `mut`, reassignment uses `=`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 3 — Grammar
- **Spec rules:** none yet (grammar chapter)
- **Evidence:** [design/syntax-measurements-r1.md](../design/syntax-measurements-r1.md#addendum-indentation-signatures-declarations)

## Context

With immutable-by-default bindings ([ADR 0038](0038-immutable-bindings-by-default.md)),
a declaration form and a mutability marker are needed. A three-line sample
measured 20 tokens with `:=`, 22 with `let`/`var`, and 23 with `let`/`let mut`
(cl100k).

## Options considered

### Option A — `x := 0`, `mut x := 0`, `x = 1` (chosen)
- **Pros:** The fewest tokens. Declaring and reassigning look different, so a
  typo can't silently create a new variable.
- **Cons:** `:=` and `=` differ only slightly visually. The compiler
  catches misuse.

### Option B — `let x = 0` / `var x = 0`
- **Pros:** Keyword-led and easy to scan.
- **Cons:** +1 token per declaration, and `let`/`var` look alike.

### Option C — `let x = 0` / `let mut x = 0`
- **Pros:** Explicit.
- **Cons:** The most tokens.

## Decision

- `name := expr` declares an immutable binding.
- `mut name := expr` declares a mutable binding.
- `name = expr` reassigns an existing mutable binding.

## Consequences

- Using `=` on an undeclared name is a compile error. The fix offered is
  "declare with `:=`".
- Using `:=` on a name already declared in the same scope is a compile error
  (no silent redeclaration). Whether shadowing in *nested* scopes is allowed is
  open (see [ai-author-wishlist.md 1.2](../design/ai-author-wishlist.md)).
- How to write an explicit type annotation on a declaration needs to be
  designed consistently with [ADR 0037](0037-signature-syntax.md).

## Rejected alternatives

- **`let`/`var`:** more tokens, and the two keywords look alike.
- **`let mut`:** the most tokens.
