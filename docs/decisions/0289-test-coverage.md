# 0289. `ail test --coverage` reports lines and functions run

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Tooling for agents
- **Spec rules:** test.cover.1-3

## Context

Agents fix what they can see. A test run says which tests pass but not which code they reach, so an agent cannot tell what is untested.

## Options considered

### Option A — per-statement line flag in the interpreter (chosen)
- **Pros:** exact for the interpreter; simple; the same hook as the run limits.
- **Cons:** line granularity; no branch coverage.

### Option B — instrument the source
- **Pros:** works for a future compiler too.
- **Cons:** a second front-end pass and a way to map back; far more code.

## Decision

Add `--coverage` and `--coverage=lines` to `ail test`: a short per-file table, or JSON lines with `--json`, built from the lines of statements that ran.

## Rationale

AI understanding: a table with ranges to read next. The flag is off by default and shares the run-limit hook, so the off path is unchanged.

## Consequences

- Agents can ask for unrun ranges and write the missing test.
- Branch and condition coverage are not covered.
- Doc-test lines map back to the file through the doc-test line map.

## Rejected alternatives

- Source instrumentation: more machinery than the interpreter needs now.
- Counting every execution: not needed to say what ran, and costs more.
