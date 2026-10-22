# ADR 0811: `ail fix` turns `for v in ch.recv()` into `for v in ch`

- Status: Accepted
- Decided by: AI (delegated, ADR 0076)
- Date: 2026-10-21

## Context

The for-loop mistakes in the corpus were three: a C-style head (fixed since
ADR 0335), `for i, x in enumerate(xs)` (fixed), and `for v in ch.recv()`
(expr.loop.4), where `recv()` gives one optional value and the loop wanted
every value until the channel closes. Only the last had no fix.

## Decision

When the iterable is `NAME.recv()` with no arguments and `NAME` is a local of
channel type, the fix replaces it with `NAME`. The loop then receives until
the channel closes (conc.share.7). Any other optional iterable keeps the
help text only: whether to loop on `while`, unwrap once or iterate something
else is the author's choice.

## Rejected alternatives

- Fixing any optional iterable by dropping the call: only the channel case has
  one meaning.
- Rewriting to `while v := ch.recv()`: it receives the same values but is
  longer, and the `for` form is what the diagnostic already recommends.
