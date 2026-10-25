# 0882. Edit impact: which tests and effects an edit touches

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076), within the item the designer approved in [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) (item 3)
- **Topic:** 22 — Tooling
- **Spec rules:** api.impact.1-4, edit.out.5, test.affected.1

## Context

After an edit an agent runs the whole test suite, or guesses which tests matter. The
checker already knows what each function names and what effects it has
([ADR 0881](0881-ail-ask.md) reads the same graph). Running only the tests a change can
reach saves time and output tokens.

## Options considered

### Option A: text diff plus the reference graph (chosen)
- **Pros:** a declaration is changed when its text differs from before (no semantic diff
  needed); the graph the checker already produces says what reaches it; one rule serves
  `ail edit --impact`, `ail ask --affected` and `ail test --affected`; conservative, so
  a test is never missed.
- **Cons:** it may run a test a change can't alter (a comment-only change is ignored, but
  a reformatted body counts as changed).

### Option B: coverage-based selection (run once, record which tests touch which lines)
- **Pros:** exact for the recorded run.
- **Cons:** needs a recorded run per test and a store that goes stale; it costs time on
  every unaffected edit and misses code a test reaches only on other inputs.

### Option C: re-run every test
- **Pros:** nothing to maintain.
- **Cons:** the cost this decision removes.

## Decision

A declaration is *changed* when it is new or its text differs from before, ignoring
blank lines, comments and attributes. A function is *affected* when it is changed, names a
changed declaration or calls an affected function; a trait implementation reaches its
trait method's callers. `ail edit --impact` reports changed declarations, affected tests
and changed effects for the edit; `ail ask PROGRAM --affected [--rev REV]` does the same
against a git revision (default `HEAD`); `ail test --affected [--since REV]` runs only the
affected tests (api.impact.1-4, edit.out.5, test.affected.1). Everything is computed only
when asked for.

## Rationale

Token usage ranks first (ADR 0076): fewer tests run, so less output to read. Conservative
reachability keeps the guarantee an agent needs: an unlisted test can't fail because of the
change. Option B's recorded runs would add cost to builds that don't use the feature,
against ADR 0868's rule that unused tools cost nothing.

## Consequences

Tests of imported modules always run under `--affected`. Selection by reachability is
coarse for large types: a change to a struct reaches everything that names it.

## Rejected alternatives

- **Coverage-based selection:** a recorded run per test that goes stale, and a cost on every edit.
- **Re-running every test:** the cost this removes.
