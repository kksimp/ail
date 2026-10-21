# 0805. The message of an error type is a method

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 23 — Compiler architecture
- **Spec rules:** [native.subset.1](../spec/23-native.md), err.msg.1, err.msg.2, trait.impl.6
- **Builds on:** [ADR 0278](0278-messages-on-error-types.md), [ADR 0804](0804-native-error-values-are-structs-and-enums.md)

## Context

The parser turns the message after `error struct Name` or a variant into an
`impl ToStr for Name` whose `to_str(self)` has the message as its body, with
the fields in scope by name. The native subset has no impl blocks yet.

## Decision

Before lowering, an impl of `ToStr` for an error type has its methods given
that type as their owner. They then declare and lower like any struct or enum
method, as `Name.to_str`, with bare field names meaning `self.field` and an
`error enum`'s message a `match self`. Nothing else about impls changes.

Printing and interpolating a struct or enum value that has a `to_str` method
call it (`display_call`), so `print(err)` and `"{err}"` give the display text
(type.str.3). Other structs still cannot be printed.

## Rejected alternatives

- **Evaluate the message at the declaration.** It reads fields, so it needs
  the value.
- **Wait for general impl blocks and traits.** Blocks every std function that
  names an error type.
