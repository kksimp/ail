# 0463. Coverage is reported per spec rule, with per-file coverage from `ail test`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy (tool)
- **Spec rules:** none (a tool)

## Context

`ail test --coverage` (test.cover.1) counts lines of one program, but most of
`compiler/` and the standard library are exercised by Rust tests that drive the
binary. The question the designer asked is whether each *rule* is tested.

## Options considered

### Option A — rule ids mentioned in tests (chosen)
- **Pros:** answers "what behaviour is untested" at the level the spec is written; a script of 70 lines;
  the same ids that diagnostics cite (ADR 0002).
- **Cons:** a mention isn't proof; a rule can be named in a test that doesn't exercise it.

### Option B — line coverage of the interpreter and checker (llvm-cov)
- **Cons:** needs a nightly or instrumented toolchain and says nothing about AIL-level rules.

## Decision

`tools/coverage/rules.py` lists the rule ids defined in `docs/spec/` that no case,
example or Rust test names (ranges such as `err.prop.1-4` count each id).
`docs/design/rule-coverage.md` is its output; the integrator files each chapter's
list to the owning lane. Line coverage of AIL programs stays `ail test --coverage`.
