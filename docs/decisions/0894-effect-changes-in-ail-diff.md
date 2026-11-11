# 0894. Effect changes in `ail diff`

- **Status:** Accepted (built by [ADR 1024](1024-ail-review.md) as a section of `ail review`)
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0396](0396-ail-diff-semantic-diff.md), [ADR 0868](0868-ai-first-tooling-after-rust-removal.md) item 3
- **Topic:** 20 — AI semantic API

## Context

`ail diff` reports changed declarations and body hashes. A reviewer, human or agent, most needs to know when a change lets code touch the network, the file system or processes, or adds failure.

## Decision

- `ail diff` gains an `effects:` section: for each function whose effect set changed, `f: +net, -fs`, and the callers that inherit the change, down to `main` and `@test`.
- A change that adds an effect to a function reachable from `main` is marked `NEW CAPABILITY` and is listed first.
- `ail diff --effects-only` prints just this section and exits non-zero with `--deny-new`, so a CI-less gate (ADR 0085) can fail a pull request that widens capabilities.
- Planned rule keys: `diff.effects.*`.

Cost when unused (ADR 0868): none. The feature is a separate command or pass, so a program that does not use it gets no extra code, no runtime check and no slower build.

## Consequences

Effects are already inferred (ADR 0868 item 1); this reads them from two program states.

## Rejected alternatives

- **A separate command:** reviewers already run `ail diff`; a second command is skipped.
- **Only direct effects:** hides the common case where a callee gains an effect and every caller inherits it.
