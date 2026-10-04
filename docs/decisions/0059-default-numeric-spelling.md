# 0059. Both `int`/`i64` and `float`/`f64` are canonical; the formatter keeps the author's choice

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** Designer
- **Topic:** 2 — Core syntax; 4 — Type system
- **Spec rules:** [type.num.4](../spec/04-types.md)
- **Resolves:** OQ-031

## Context

[ADR 0055](0055-numeric-type-names.md) introduced `int` (= `i64`) and
`float` (= `f64`). Canonical syntax normally requires a single spelling.

## Options considered

- **Keep the author's choice (chosen):** preserves intent. `int` says "width
  doesn't matter" and `i64` says "exactly 64 bits matters". Con: a deliberate
  exception to "one canonical form".
- **Always `int`/`float`:** loses the "width matters" signal.
- **Always `i64`/`f64`:** makes `int` pointless input-only sugar.

## Decision

The formatter never rewrites between `int` and `i64`, or between `float` and
`f64`. Both spellings are canonical. They denote identical types.

## Consequences

- This is a **documented exception** to canonical-syntax principle #7.
  Further exceptions need their own ADRs.
- Diagnostics and the AI API show the spelling used at the declaration site.

## Rejected alternatives

- **Forced single spelling:** loses author intent that's useful to readers.
