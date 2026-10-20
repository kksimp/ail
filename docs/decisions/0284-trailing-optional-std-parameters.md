# 0284. A standard function's trailing optional parameters may be left off

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard modules; functions
- **Spec rules:** std.proc.2

## Context

ADR 0282 adds `input` and `timeout_ms` to `proc.run`. Named arguments
are not in the bootstrap yet (ADR 0058), and default parameter values are
not designed for user functions.

## Options considered

### Option A — the standard library may declare trailing `T?` parameters that callers omit (chosen)
- **Pros:** `proc.run("ls", [])` keeps working; `proc.run("sort", [], text)`
  and `proc.run(cmd, args, none, 5000)` read positionally; no new syntax.
- **Cons:** an arity rule user functions don't have yet.

### Option B — separate functions (`run_with`, `run_input`)
- **Cons:** more names to learn and choose between; combinations multiply.

### Option C — wait for default parameter values in the language
- **Cons:** a core-language design question (hard to reverse) for one
  standard function tonight.

## Decision

A standard function may have trailing optional (`T?`) parameters that a
call leaves off from the end; an omitted one is `none`. Today only
`proc.run` has them. Arity errors report the full count.

## Rationale

Criterion 1 (tokens) and 2 (one name per operation). The rule is confined to
the standard library, so it doesn't decide the user-facing design of
defaults, which stays open.

## Consequences

- If user functions get defaults later, this becomes a special case of that
  rule.

## Rejected alternatives

- More function names (Option B): ambiguity and card tokens.
- Designing defaults for all functions tonight (Option C): too large and
  hard to reverse for this lane.
