# 0631. `Json.to_text` prints whole numbers without a fraction

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 17 — Standard library
- **Spec rules:** json.print.1 ([std-json](../spec/std-json.md))
- **Amends:** [ADR 0161](0161-std-json-in-the-prelude.md) (the number text)

## Context

`S(n: 3).to_json().to_text()` gave `{"n":3.0}`: every `Json.Num` is a
`float` and printed like one. An `int` field written through `ToJson` and
read by another program (Python's `json.loads`, `jq`) came back as the float
`3.0`, and `"hours":2.0` costs a token more than `"hours":2`. Found by lane
228 twinning `examples/config`.

## Decision

A `Num` that is finite, whole and of magnitude below 1e15 prints as an
integer (`3`, `-12`, `0`; a zero's sign is dropped). Every other number
prints as the `float` does (`2.5`, `1e+20`); infinite and NaN stay `null`
(json.print.2). The parser is unchanged, so `parse_json("3").to_text()` is
`3` and the round trip law of json.print.2 still holds.

Criteria: tokens (shorter text on every whole number) and AI understanding
(output matches what the writer meant and what other tools produce).

## Consequences

Expected outputs of 13 cases and examples that print JSON changed from
`3.0` to `3`; no test was weakened. Plain `print(3.0)` is untouched.

## Rejected alternatives

- **A separate `Json.Int` variant.** The right long-term model, but it
  changes the `Json` enum every program matches on; not a bug fix.
  Recorded as a possible later change.
- **Printing by field type only in derived `to_json`.** The text is made
  from `Json`, which carries no int/float distinction, so the printer has to
  decide.
- **Threshold 2^53 instead of 1e15.** 1e15 keeps the integer text exact on
  every consumer and well inside `to_int`.
