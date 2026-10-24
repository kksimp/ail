# 0879. `@observe` and live attach: read-only program state for agents

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, "Approve; attach on in ail run")
- **Topic:** 20 — AI semantic API; std.app
- **Builds on:** [ADR 0868](0868-ai-first-tooling-after-rust-removal.md), [ADR 0878](0878-screen-model-for-std-app.md)

## Context

Agents need to know what a GUI program shows and what state it holds without a vision
model or scraping. Garlic's AI-first plan (designer, 2026-10-23) proposed both. Neither
may cost anything when no agent is attached.

## Decision

1. `@observe` on a field of a program's state (`@observe cart_total int`) makes it a named, typed value that an attached agent can read and watch. It is the program telling agents its state directly, instead of agents scraping pixels.
2. Live attach: a running native program answers `observe`, `screen` and heap-summary queries over a local socket. The token file is mode 0600, and nothing listens on the network. This is ADR 0868 item 8.
3. Read-only. Agents read and watch; they never write a program's state through attach. Changing state still goes through the program's own actions, such as a button or an MCP tool the program defines.
4. Cost: the attach listener and the observe table are compiled in only when attach is on. With attach off there is no code and no thread.
5. Attach is on by default for `ail run` (development) and off for `ail build` unless `--attach` is given.

## Rejected alternatives

- A vision model over screenshots: slow and token-heavy, and it guesses what the program already knows.
- Streaming every frame: costs tokens whether or not anyone asks.
- Writable observe: a second, unchecked way to change a program's state.
