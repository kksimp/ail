# 1091. `std.mcp`: a client for MCP servers

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.mcp.1-8 ([17-std](../spec/17-std.md))

## Context

The AI-first review (item 17) found that an AIL program can serve tools over MCP
(`ail mcp`, ADR 0918; `@tool`, ADR 0986) but cannot use any: the agent ecosystem
is MCP servers, and a program that drives one hand-writes the JSON-RPC framing.

## Decision

`std.mcp`, written in AIL (`std/mcp/mcp.ail`): `connect(cmd, args)` starts a
server over stdio with `proc.start`, does the `initialize` handshake and returns
a unique `Client`; `tools()` lists tools (all pages); `call(name, Json)` returns
`structuredContent` or the result; `mcp.text` reads text blocks. Failures are an
`McpError { kind, message }` with four kinds. Messages that are not the pending
reply are skipped; each reply waits 30 s by default. No runtime, lowering or
Rust change: only the four std registration lines.

## Rejected alternatives

- **Typed result variants** (text, image, resource). Cost: a type per content
  kind before programs ask for them; the `Json` result already carries them.
- **HTTP and SSE transports.** Cost: TLS client sessions and streaming parsers
  for a first version; stdio is how local servers run. Add for 0.2.
- **Server-side features** (resources, prompts, sampling). Not needed to call tools.
- **A hard dependency on a public server in tests.** The test runs a server
  written in AIL, so no network is needed.

## Measurements

None: tokbench has no MCP twin.
