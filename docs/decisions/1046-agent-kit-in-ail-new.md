# 1046. `ail new` writes an agent kit

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076). Reasoning: token usage (an agent that learns the loop from 350 tokens needs fewer exploratory rounds), then AI understanding.
- **Topic:** 22 — Tools
- **Spec rules:** [tool.new.3, tool.new.4, tool.new.5, task.finish.3](../spec/22-tools.md)
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

## Amendment 2026-11-11: join-up with the tools that landed after it

Decided by: AI (delegated, ADR 0076), gap G4 of the final-gaps review.

1. The kit text loads the 970-token core card (`ail card`, ADR 1051) and looks up the rest with
   `ail card TOPIC` and `ail ask std WORDS`, not the 5,600-token full card. The loop gains
   `ail check --fix` (ADR 1047) and `ail try` (ADR 0893) for two candidate fixes; `ail task finish`
   writes the change certificate (ADR 1052). Still at most 350 tokens.
2. `ail new .` (tool.new.5) writes only the kit into a project that already has `.ail` files or
   `ail.pkg`. It never overwrites a file and never appends to `AGENTS.md`; it prints the block to add.
3. `ail task finish` runs `ail verify --diff --rev REV --attest .ail/cert.json PROGRAM` (task.finish.3). A
   failing certificate keeps the task open and exits 1. `ail verify` leaves `.ail-task` and `.ail/` out of
   the certificate's `files`, so `ail verify --check` still reproduces it after the task closes.
4. The benchmark arm `ail-tools` carries this text, and a new arm `ail-core-card` loads only `ail card`
   (ADR 0916, amended): the paid run measures what the kit now says.

Rejected: appending to an existing `AGENTS.md` (an edit to a file the user owns); a `--force` flag on
`ail new .` (the printed block is enough).

## Rejected alternatives

- **Instructions only in the language card.** The card is already the largest token cost.
- **A hosted docs site.** ADR 0003: GitHub and the wiki only.
- **Opt-in `--agent`.** The agents that need the kit are the ones that would not know the flag.

## Consequences

Every new project carries two small files; `--no-agent` removes them. The kit is measured by the
`ail-tools` benchmark arm (E3 of the AI-first edge plan).
