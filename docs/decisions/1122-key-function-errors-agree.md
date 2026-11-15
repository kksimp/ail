# 1122. Key-function errors agree: typed `by: |x T|`, no type.fn.4 after a failed receiver

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); slice S4 of the E13 follow-up, direction approved by the designer on 2026-11-15
- **Topic:** 5 — Expressions; 14 — Tooling (diagnostics)
- **Spec rules:** expr.key.1, expr.key.2, type.fn.4
- **Builds on:** [ADR 0250](0250-key-functions-and-take.md), [ADR 1067](1067-closures.md)

## Context

In the E13 agent benchmark, 10 AIL trials spent two fix rounds each on one line,
`idx.sorted(by: |i| (-xs[i], i))`, after an earlier line had failed (often
`idx := (0..n).to_list()`). Two rules then gave opposite fixes:

1. The receiver's type was unknown, so the checker checked the argument with no
   context and `type.fn.4` said "the type of `i` isn't known here; write
   `|i int| ...`".
2. With `|i int|` written, the parser no longer saw the key-function form
   `|x|`, and `expr.key.1` said "`by:` takes a key function; write
   `by: |x| key`".

`expr.key.1` also used one message, "takes no `by:` argument here", for five
different mistakes (wrong method, wrong receiver, extra arguments, a missing or
wrong label), and reported a misplaced `|x|` twice.

## Options considered

### Option A — accept `|x T|` and silence the follow-on error (chosen)
- **Pros:** both fixes the agent tried now compile; the follow-on error is gone,
  so the first error is the only one; no new syntax.
- **Cons:** two spellings of a key function.

### Option B — keep `|x|` only and make type.fn.4 point at `|x|` inside `by:`
- **Pros:** one spelling.
- **Cons:** an agent that writes the type (the form every other function
  literal needs without context) still pays a round; the written type is
  harmless and checkable.

### Option C — treat `by:` arguments as ordinary function values `fn(T) K`
- **Pros:** one model for literals and keys; named functions would work.
- **Cons:** changes ADR 0250's "a key function isn't a value" and the inline
  lowering (no closure object, ADR 0571); a language change, not a diagnostic
  fix.

## Decision

- `by: |x T| key` is a key function; `T` MUST be the element type
  (`expr.key.2`), and a different type is one error naming both types.
- A function literal among the arguments of a method call whose receiver's type
  is unknown (an error was already reported) gets no `type.fn.4` error for a
  parameter without a type.
- Each `expr.key.1` error names what is wrong with the argument written: the
  method takes no key, the receiver isn't a list or set (with a fix for a map or
  a set's `sort`), extra arguments, a missing or wrong label, a literal with
  other than one parameter, or the type of a value given instead.

## Rationale

AI understanding first (ADR 1077): one mistake gives one error, and no error's
fix triggers another error. Readability is unchanged; `|i int|` reads as well
as `|i|`. No runtime cost: the parser fills the same key-function form, and the
lowering is unchanged.

## Consequences

- `xs.sorted(by: |i int| (-ys[i], i))` compiles; tuple keys were already
  ordered (`type.tuple.7`) and sorted natively.
- The checker gains `compiler/check/keyfn.ail`; `LitState.after_error` counts
  the calls whose arguments are checked after an error.
- Card line for the sync slice: "`by: |x| key` or `by: |x T| key`; a tuple key
  `(a, b)` orders by `a`, then `b`."

## Rejected alternatives

- Option B: still costs a round whenever the type is written.
- Option C: reverses ADR 0250 and ADR 0571 for a diagnostic problem.
