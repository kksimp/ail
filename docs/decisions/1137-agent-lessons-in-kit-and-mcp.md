# 1137. The kit and the MCP server open with the same two lessons

- **Status:** Accepted
- **Date:** 2026-11-15
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); asked for by the designer after the AI-first tooling panel (D4, D5)
- **Topic:** 22 — Tools
- **Spec rules:** [tool.new.3, mcp.proto.4](../spec/22-tools.md)
- **Builds on:** [ADR 1046](1046-agent-kit-in-ail-new.md), [ADR 0918](0918-ail-mcp-compiler-tools.md); the fixes it names are ADR 1130 (in flight)

## Context

In the E13 archive, `ail card` read in full before the first draft gave 0.97 failed builds per new program; the
core card plus 14 lookups gave 2.51. The agent file told agents to run `ail card` once, and 48 of 96 did. Lookups
(`ail ask`) came before the first draft and asked for library names, never for a form the agent believed it knew.
MCP was never used by agents with a shell, and `initialize` sent no `instructions`. Text that arrives unasked, at
the start of a session, is the only channel the measurements show agents obeying.

## Decision

1. `AGENTS.md` from `ail new` opens with two lessons, before the tool lines: read the whole card once before
   writing code, naming its blocks "Habits from other languages" and "Compiles, but is not what you meant" (no
   diagnostic reaches the second, so the agent checks it); and `ail check`, `build`, `run` and `test` rewrite some
   wrong spellings and print `fixed ...`, so the agent reads the file again before editing it.
2. `ail mcp` answers `initialize` with an `instructions` field: one line naming the tools, then those same two
   lines (mcp.proto.4).
3. The lines live twice, in `compiler/scaffold/scaffold.ail` and `tools/ailmcp/ailmcp.ail`, because the MCP server
   is a separate program that runs `ail` as a subprocess and links no compiler module. The `agent_tools` gate suite
   fails if the `AGENTS.md` lines are not inside `instructions`, so they cannot drift.
4. The kit's cap rises from 350 to 475 tokens (about 1,900 bytes in the gate). The lessons add about 120 tokens;
   the other lines are unchanged. Under ~2% of an agent session's tokens, and measured against the first-draft
   failures they remove, a tie on tokens, so AI understanding decides.

## Rejected alternatives

- **Generate the lines from the card at build time.** The card is rewritten often (ADR 1110) and its blocks are
  prose with examples; a generator has to pick lines, and a wrong pick is silent. A gate comparison of two short
  lists fails loudly instead.
- **A subprocess call to `ail` for the text on `initialize`.** Costs a process start per session for two lines.
- **Lessons in tool descriptions.** `tools/list` is held under 1,500 tokens (mcp.tools.3), and an agent reads a
  tool description only when it considers that tool.
- **Repeat the card's habit lines in the kit.** Pays the card's tokens twice; the card is read anyway.
