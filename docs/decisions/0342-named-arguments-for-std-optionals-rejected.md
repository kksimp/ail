# 0342. Named arguments for the optional parameters of std functions: not now

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Std, token efficiency
- **Spec rules:** none

## Context

OQ-112 proposed `proc.run(cmd, args, timeout_ms: 200)` in place of
`proc.run(cmd, args, none, 200)`, estimating a saving of 1–2 tokens per call.
ADR 0058 leaves named arguments unsupported for functions in the bootstrap.

## Options considered

### Option A — accept `input:` and `timeout_ms:` on `proc.run` (built, measured, reverted)
- **Pros:** names the argument, as Python's `timeout=` does.
- **Cons:** measured **+1.4 tokens** on `runner` (291.2 to 292.6), since
  `timeout_ms:` costs more than `none,`. A shorter `timeout:` ties it exactly
  (291.2), so no spelling saves anything.

### Option B — leave it (chosen)
- **Pros:** no new call form; `none, limit` costs what the named form costs.

## Decision

Don't add named arguments to std functions. `proc.run(cmd, args, none, limit)`
stays.

## Rationale

Tokens decide first (ADR 0076) and the measurement is a loss or a tie. The
only other gain is readability for a call that appears once per program.

## Consequences

- Resolves OQ-112 as "leave it". Revisit if a std function grows several
  optional parameters where `none, none, x` would be common (`a.load(order:)`
  in OQ-097 is the next candidate).

## Rejected alternatives

- Option A: it is more tokens than the form it replaces.
