# 0278. Error types carry their messages

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Errors; token efficiency
- **Spec rules:** err.msg.1–2

## Context

OQ-072: `sheet` and `json` spend 115 tokens on `impl ToStr` blocks whose only
job is one message per error variant. The message sits far from the variant
it describes, behind `impl`, `fn to_str(self) str` and a `match` that repeats
every variant and its fields. OQ-072 recommended waiting until error-heavy
programs joined the corpus; round 5 measured it again.

## Options considered

### Option A — a message after the variant or the struct's name (chosen)
`BadName(text str) "bad cell name '{text}'"`; `error struct ParseError
"parse error at {line}:{col}: {msg}"`. The message interpolates the fields by
name and defines `impl ToStr`.
- **Pros:** −115.4 tokens on the examples (−0.63%; `sheet` −1.6%); the
  message is next to the data it shows (local reasoning); the form agents know
  from Rust's `thiserror`.
- **Cons:** a second way to give a type its display text.

### Option B — keep `impl ToStr`
- **Pros:** one mechanism.
- **Cons:** the message is far from its variant and costs a `match`.

## Decision

An `error struct` may have a message after its name, and every variant of an
`error enum` may have one after its fields: a string literal that reads the
fields by name. Messages define the type's `impl ToStr`. Every variant has one
or none does; generic error types and non-error types can't have one. An
`error struct` with a message may have no fields.

## Rationale

Tokens are a tie on the agent corpus (0%) and on the examples (−0.63%), so AI
understanding decides (criterion 2): the message sits on the variant's own
line, so reading a variant tells what it prints, and the `match` that repeated
every variant is gone. The bootstrap turns the messages into an ordinary
`impl ToStr`, so `ail api`, display and `?` reporting need no new rules.

## Consequences

- `sheet` and `json` use messages; the prelude's `Absent` (ADR 0279) is a
  field-less error struct with a message.
- Lane 55's error-message work builds on this form.

## Rejected alternatives

- Keeping only `impl ToStr` (B): the message stays away from its variant.
- A message on some variants with a default for the rest: two places to look
  for one type's text.
- `#[error("...")]`-style attributes: an attribute with an argument costs more
  tokens than the bare string and adds a new attribute.
