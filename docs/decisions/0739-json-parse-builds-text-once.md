# 0739. The native JSON parser builds each text value once

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 13 — Standard library
- **Spec rules:** json.parse.1, json.parse.2 (no rule changes)
- **Builds on:** [ADR 0658](0658-native-json.md), [ADR 0737](0737-runtime-small-block-allocator.md)

## Context

`json_roundtrip` is still about 2x CPython after ADRs 0658 and 0737. The
native parser built every text value in three steps: a `String` for the
characters, a copy into an `Rc<str>`, and the free of the `String`. A
document is mostly text (every key and every string value), so that was most
of the parser's allocation.

## Decision

`Parser::string` returns an `Rc<str>`. A string with no escape is made
straight from the input slice; one with an escape is assembled in a `String`
as before. Values, errors and offsets are unchanged.

## Measured (callgrind, `json_roundtrip` with 2,000 records, instructions)

| | before | after |
|---|---|---|
| whole program | 77.2 M | 74.2 M (-3.9%) |

The rest is the AIL side: building the records (`op_build`), `Json.num`,
`bool` and `as_arr` calls, which run in the bytecode tier and are not touched
here. `./ail selftest` passes.

## Rejected alternatives

- **Appending runs in `quote`.** Keys and values are short, and the run
  bookkeeping cost more instructions than it saved (74.5 M).
- **Native `Json.num`, `bool` and `as_arr`.** ADR 0658 judged them one or two
  instructions per call; the per-call cost is in the tier.
- **A flatter `Value::Enum` (no `Rc<Vec>`).** It changes every pattern match
  on enums and structs in the interpreter; a separate decision.
