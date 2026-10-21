# 0804. Native error values are ordinary structs and enums

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), err.decl.1, err.msg.1, err.ret.1
- **Builds on:** [ADR 0576](0576-native-error-values-messages.md) (its error value is replaced here), [ADR 0575](0575-native-results-error-slot.md), [ADR 0392](0392-native-structs-and-methods.md), [ADR 0602](0602-native-enum-layout.md)

## Context

ADR 0576 accepted only `error struct`s without fields and made the error
value its message text. Structs and enums now have layouts (kind 4 and 5), and
the standard library names error types on its public functions (err.type.3),
so native code needs error structs and enums with fields, and `else err` that
can read them.

## Decision

An `error struct` and an `error enum` are ordinary structs and enums with the
same layout; `err_msgs` only marks their names as error types. A failure keeps
two things in the runtime: the **display text** in global 8 (as before, so the
report and the trace do not change) and the **value** in global 12 (`ERRV`):
the struct or enum pointer, or the tag of a fieldless enum.

`return E(...)`, `return E.V`, `return E.V(...)` and a bare variant of an
error enum, in a function that can fail, build the value, call its `to_str`
for the text, and store both (`rt.err_set`, `rt.err_value`). `rt.err_clear`
and `rt.err_take` clear both. The collector scans the globals, so the value
stays alive while it travels.

## Consequences

- The text is built when the error is raised, not when it is reported; the
  failure path pays one `to_str` call.
- `fail` through a union or an inferred set keeps working through the text.

## Rejected alternatives

- **Build the text only in the report.** Needs the type at the report; a
  union has no single type there.
- **Box value and type together.** More layout than a typed `else err` needs
  (ADR 0806).
