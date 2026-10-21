# 0576. Native error values: messages only, for now

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), err.decl.1, err.msg.1, err.else.3
- **Builds on:** [ADR 0575](0575-native-results-error-slot.md)

## Context

Error values are structs and enums, which are lane 120's critical path
(kind 4 and kind 5 objects, `match`). Lane 175 needs errors now to test `?`
and `else`.

## Decision

This slice accepts `error struct Name "message"` with **no fields** (and no
generics). The error value is the message's string object, so `else err`
binds a `str` and `{err}` prints the display text. Anything that needs the
type of the error (a field, `match err`, an `error enum`, a message with
`{field}`) stays outside the subset and is reported by native.subset.1.
When lane 120's struct layout lands, an error value becomes the struct
pointer and `rt.err_set` takes it with a display function; the call sites
in `lower_err.ail` change in one place (`error_ctor`, `lower_else`'s
binding).

The `to_str` the parser generates from a message is recorded and skipped
rather than reported as a method.

## Rejected alternatives

- **Wait for structs.** Blocks every test of `?` and `else` behind lane 120.
- **A tagged error word** (type id and payload): more design than the
  struct layout will need.
