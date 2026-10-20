# 0190. Moves are checked on every path; only owned values move; `move` moves any type

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 7 — Ownership
- **Spec rules:** [own.move.2–5](../spec/07-ownership.md#copies), own.uniq.2

## Context

ADRs 0066 and 0068 say a moved-from binding "can't be used afterwards", but
not what "afterwards" means with branches and loops, which values may be
moved, or whether `move` applies to copyable values (ADR 0068 left that
open). The bootstrap needs exact answers to implement moves.

## Options considered

### "Afterwards"
- **Every path (chosen):** a use is an error if any path reaches it after a
  move; a move inside a loop of an outer binding is an error unless the path
  leaves the loop or reassigns the binding. Errors name the move as a note.
  Con: a flow analysis in the checker.
- **Textual order:** a use later in the source is an error. Simple, but
  misses loops (the next iteration) and rejects `if c: close(move f) else: use(f)`.
- **Run-time check:** a moved binding traps when used. Cheap to build, but a
  bug class the checker can rule out.

### What can be moved
- **Owned values only (chosen):** a local binding, a `move` parameter, or a
  new value. Parameters without `move`, `self`, loop variables and parts of
  values (`w.f`) can't be moved. Con: moving out of a field needs a
  destructuring form, not designed yet.
- **Any place:** allows `move w.f`, but leaves `w` half-moved, which every
  later use of `w` must then track.

### `move` with a copyable type
- **It moves (chosen):** the binding ends, no copy is made, and there is no
  warning. One rule for every type, and the hand-over of storage is explicit
  (the in-place reuse ADR 0068 mentions).
- **Warn and copy:** gives `move` two meanings depending on the type.
- **Reject:** forbids handing over a large buffer without a copy.

Also decided: an argument marked `move` to a parameter that isn't `move` is
an error, like `mut` (ADR 0061).

## Decision

Uses after a move are checked on every path (own.move.4); only owned values
can be moved (own.move.3); `move` moves values of any type (own.move.5).

## Rationale

AI understanding decided (criterion 2). Tokens are equal across the options;
each choice keeps one meaning per construct and gives a compile-time error
with both positions instead of a run-time failure. The flow analysis is
small because names are never shadowed (ADR 0044) and borrows are
second-class (ADR 0062).

## Consequences

- `bootstrap/src/check/moves.rs` walks each function once after type
  checking; the checker records each moving use in `Resolved::moves`, which
  the interpreter also uses to take the value out of its binding.
- Moving out of a field, and `move self` receivers, are open (OQ-050).

## Rejected alternatives

- **Textual order:** unsound in loops and too strict across branches.
- **Run-time check:** turns a checkable error into a trap.
- **Moving any place:** half-moved values complicate every later use.
- **Warn and copy / reject for copyable types:** two meanings, or a lost
  optimisation.
