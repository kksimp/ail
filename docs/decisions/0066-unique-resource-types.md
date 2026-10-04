# 0066. Resources use non-copyable (unique) types released at scope end, plus `defer`

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 7 — Ownership; 22 — Runtime; 21 — Standard library
- **Spec rules:** [own.uniq.1–4, own.defer.1–2](../spec/07-ownership.md)
- **Resolves:** OQ-034

## Context

Memory is reclaimed by the GC and the compiler, but files, sockets and locks
must be released promptly and predictably. GC finalizers can't guarantee that.

## Options considered

- **Unique types, scope-end release, and `defer` (chosen):** release at a
  predictable point, and forgetting to close is impossible. Con: a new
  concept (moves become visible for these types), and scope-end timing.
- **`defer` only:** easy to forget, with no guarantee.
- **`with` blocks:** an extra indentation level, and still optional.

## Decision

1. A type may be declared **unique** (marker syntax to be decided;
   `unique struct` is illustrative). Values of a unique type can't be
   copied, only moved.
2. Using a unique value as an argument to a parameter that takes ownership,
   or assigning or returning it, **moves** it. The source can't be used
   afterwards (a compile-time error).
3. A unique type may define a cleanup operation. It runs deterministically
   when the value's current owner goes out of scope, if the value hasn't
   been moved away. That includes scope exit on a trap.
4. `defer statement` schedules a statement to run when the enclosing block
   exits by any path, including traps. Multiple defers run in reverse order.

## Consequences

- GC finalizers are not part of the language.
- Parameter modes for unique values need an explicit "take ownership" form
  in addition to read-only and `mut` borrows. Syntax to be decided (OQ-035).
- Unique values can't be put in `ref` cells or other shared structures
  without explicit rules. To be designed.
- Cleanup order within a scope is reverse declaration order (to be
  specified with the full cleanup rules).

## Rejected alternatives

- **`defer` only, `with` blocks:** release isn't guaranteed.
- **Finalizers:** non-deterministic.
