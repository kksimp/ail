# 0184. Keep `:=`, `fn`, explicit `self`, return types and `use` lines after round-3 measurement

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Syntax; token efficiency
- **Spec rules:** none (no change)

## Context

The designer allowed design changes that cut tokens where AIL doesn't beat
Python (2026-10-20), staying as close to the existing design as possible.
`tools/tokbench/attribute.py` attributed AIL's tokens to construct classes
across the eight corpus programs, and each candidate below was applied by
script to all of them (base 15,148.6 tokens, mean of five tokenizers, after
ADRs 0182–0183). Details in [program-measurements-r3.md](../design/program-measurements-r3.md).

## Options considered

| Change | Tokens | Change |
|---|---|---|
| `=` instead of `:=` for declarations ([ADR 0039](0039-declaration-syntax.md)) | 15,148.6 | 0.00% |
| No `fn` keyword | 14,969.4 | −1.18% |
| Implicit read-only `self` (`fn T.m()`) | 15,069.8 | −0.52% |
| Implicit `self` with `mut` too (upper bound, no marker) | 14,915.8 | −1.54% |
| Return-type inference for functions (keeping the `!` marker) | 14,872.8 | −1.82% |
| 2-space indentation | 15,128.0 | −0.14% |
| Tab indentation | 15,751.8 | +3.98% |
| One-line `use std.env, fs, proc` | 15,143.4 | −0.03% (−1.6% of `tool`) |
| No `use std.x` lines (standard modules always in scope) | 15,135.0 | −0.09% (−4.3% of `tool`) |

## Decision

Change none of these.

## Rationale

Every candidate is under ADR 0076's ~2% threshold across the corpus, so
tokens tie and **AI understanding** decides:
- `:=` and `=` are each one token; `:=` keeps declaration and assignment
  apart, which ADR 0039 chose it for.
- `fn` anchors every declaration for search (`grep "fn name"`) and for the
  parser's recovery; dropping it saves 1.2%.
- An explicit `self` or `mut self` tells the reader, at the signature,
  whether a function is a method and whether it changes its receiver; the
  realistic saving with a `mut` marker is about 1%.
- Return types are what `ail api` and a reader use to reason about a call
  without reading its body; inferring them saves 1.8% but loses local
  reasoning and makes errors appear far from their cause.
- Indentation width is a tie, and tabs cost 4% more.
- `use std.x` lines cost 4.3% of a 34-line tool but 0.1% of the corpus. The
  qualified access (`fs.read_text`) already names the module at each use, so
  the line is the cheapest candidate for a later change; it is left to the
  user-modules work (lane 3), which owns name resolution for modules, and is
  recorded as OQ-071.

## Consequences

- No syntax change. The measurements are repeatable with the scripts in
  `tools/tokbench/`.

## Rejected alternatives

- Each change in the table, for the reason in its bullet above.
