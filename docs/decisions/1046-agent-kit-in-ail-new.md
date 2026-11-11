# 1046. `ail new` writes an agent kit

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076). Reasoning: token usage (an agent that learns the loop from 350 tokens needs fewer exploratory rounds), then AI understanding.
- **Topic:** 22 — Tools
- **Spec rules:** [tool.new.3, tool.new.4](../spec/22-tools.md)
- **Builds on:** [ADR 0903](0903-ail-new.md) (`ail new`), the `ail mcp` server, `ail task` and `ail context`

## Context

A fresh AIL project gave an agent nothing: no agent file, no MCP registration. The agent had to
find the card, the task loop and `ail explain` on its own, or never used them.

## Decision

1. `ail new NAME` also writes `AGENTS.md` (at most 350 tokens by tokbench) and `.mcp.json`
   (`ail mcp` on stdio). `AGENTS.md` names the card (`ail api --card .`), the loop
   (`ail task start`, `ail context FN`, edit, `ail task verify`, `ail task finish`),
   `ail explain RULE` for diagnostics, and `ail ask` instead of grep.
2. `ail new --no-agent NAME` writes neither file.
3. `llms.txt` at the repository root lists the docs an agent should load, the card first.
   `README.md` carries the same instructions for existing projects.

## Rejected alternatives

- **Instructions only in the language card.** The card is already the largest token cost.
- **A hosted docs site.** ADR 0003: GitHub and the wiki only.
- **Opt-in `--agent`.** The agents that need the kit are the ones that would not know the flag.

## Consequences

Every new project carries two small files; `--no-agent` removes them. The kit is measured by the
`ail-tools` benchmark arm (E3 of the AI-first edge plan).
