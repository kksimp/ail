# 0317. `move self` receivers consume the receiver; the call site is unmarked

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 7 — Ownership
- **Spec rules:** [own.move.6](../spec/07-ownership.md#copies), own.move.3

## Context

ADR 0190 left `move self` open (OQ-050): a method couldn't consume its
receiver, so a resource's "finish" operation (`conn.close()` that returns a
status, `builder.build()`) had to be a free function taking `move c Conn`.
`move` arguments are marked at the call site (own.move.2); `mut self`
receivers are not (`c.send(10)`). Which applies to `move self`?

## Options considered

### Call site unmarked: `c.close()` (chosen)
- **Pros:** fewest tokens; same as `mut self`, whose receiver is also
  unmarked; the checker reports any later use with the move as a note
  (own.move.4), so nothing is silently lost.
- **Cons:** the move isn't visible at the call without the signature
  (`ail api` shows `move self`).

### Call site marked: `(move c).close()`
- **Pros:** every move visible locally.
- **Cons:** +2.9% tokens on the measured program; the parenthesised form
  reads poorly and differs from `mut self`.

### No `move self`; a free function `close(move c Conn)`
- **Pros:** nothing new.
- **Cons:** +3.4% tokens; the method can't use the implicit-`self` field
  names (ADR 0141), and resource APIs split between methods and functions.

Measured with `tools/tokbench/run.sh` on a 27-line connection-pool program
(`bootstrap/tests/cases/own_move_self.ail` written each way), mean over five
tokenizers: unmarked 180.0, marked 185.2, free function 186.2. Existing
examples and the agent corpus have no unique types, so they don't change.

### Also decided
- Inside the method `self` is owned like a `move` parameter: it can be
  moved on, and it is released when the method returns unless moved.
- Calling it on a part of a value (`xs[i].close()`, `w.c.close()`) is
  own.move.3, as for `move` arguments; the help names `pop`/`remove_at`.
- Not allowed in a `trait` or `impl` block yet: trait signatures compare
  receivers as `self`/`mut self` only, and a consuming trait method needs
  generic code over unique types (OQ-050).
- `move self` is read-only, like a `move` parameter; bind it to a `mut`
  local to change it.

## Decision

`fn T.m(move self, ...)` takes ownership of the receiver; `x.m()` moves
`x` with no mark at the call.

## Rationale

Tokens decided (criterion 1): 2.9% over the marked form, above the 2% tie
threshold. AI understanding is kept by the flow check, which names the move.

## Consequences

- Parser accepts `move self`; `Receiver` gains `moved`; `ail api` shows it.
- The checker records the receiver in `Resolved::moves` (rule own.move.6),
  so the interpreter takes it out of its binding and `moves.rs` checks later
  uses; the interpreter releases `self` when the method returns.

## Rejected alternatives

- **Marked call site:** more tokens, and inconsistent with `mut self`.
- **Free functions only:** more tokens, and splits resource APIs.
