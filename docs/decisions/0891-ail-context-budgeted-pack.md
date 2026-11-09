# 0891. `ail context SYM --budget N`: the smallest useful context pack

- **Status:** Accepted (built and amended by [ADR 1020](1020-ail-context.md))
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0868](0868-ai-first-tooling-after-rust-removal.md), [ADR 0394](0394-token-budgets-on-every-command.md), [ADR 0880](0880-budgeted-diagnostic-forms.md)
- **Topic:** 20 — AI semantic API

## Context

An agent that edits `f` today opens the file, then callers, tests and the effects of callees, one file at a time. Most of those tokens are wasted. The compiler already holds the graph (`ail api`, `ail ask`).

## Decision

- `ail context SYM --budget N` prints one pack that fits N tokens (counted as `--budget` counts, ADR 0394).
- Contents, in the order they are dropped last to first: the symbol's signature and `##` doc; its body; its effects (ADR 0868 item 1 form); direct callers' signatures; tests that reach it; spec rule IDs its diagnostics cite; callees' signatures.
- Each section is cut whole, never mid-item. The last line says what was dropped and the budget needed to include it.
- Output is deterministic (stable order, one path spelling) and available as `--json`.
- With no `--budget` the default is 2,000 tokens.
- Planned rule keys: `context.pack.*`, `context.budget.*`.

Cost when unused (ADR 0868): none. The feature is a separate command or pass, so a program that does not use it gets no extra code, no runtime check and no slower build.

## Consequences

The tool reads the same graph as `ail ask`; it adds no new analysis. Section ordering needs a tokbench check on a fixed set of edit tasks.

## Rejected alternatives

- **A raw file dump around the symbol:** includes unrelated code, and the cost grows with file size.
- **Callers by text search:** misses methods and trait calls that the graph resolves.
- **A fixed pack with no budget:** cannot trade completeness for tokens, which is the agent's real choice.
