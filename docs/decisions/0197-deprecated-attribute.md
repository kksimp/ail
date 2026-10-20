# 0197. `@deprecated("reason")` warns at each use

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax
- **Spec rules:** [gram.attr.4](../spec/03-grammar.md)
- **Evidence:** deciding criterion: AI understanding. A warning that carries the reason tells an AI agent what to call instead, without a doc lookup.

## Context

Libraries need to retire functions without breaking callers, and the standard
modules will grow (lane 4). `gram.attr.1` allows `@name(arguments)`, but the
bootstrap skipped every argument.

## Options considered

### Option A — warning with a required reason (chosen)
- **Pros:** the program still runs; the message says what to do; `ail check --json`
  carries it to tools.
- **Cons:** warnings are easy to ignore in a long build.

### Option B — error
- **Cons:** breaks callers the moment a library deprecates something.

### Option C — optional reason
- **Cons:** a deprecation with no replacement named is the least useful kind.

## Decision

`@deprecated("reason")` on a function makes each call of it a warning (`gram.attr.4`):
`` `name` is deprecated: reason ``. The reason is one string literal and is
required; `@deprecated` without it is an error. The warning is reported for
function, method and associated-function calls.

## Consequences

- Only string arguments are parsed so far; other attributes' arguments are still skipped.
- No `--deny warnings` switch exists yet.

## Rejected alternatives

- **B** and **C**: see above.
