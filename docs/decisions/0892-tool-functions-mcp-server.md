# 0892. `@tool` functions served by `ail serve --mcp`

- **Status:** Deferred past 0.1.0
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the feature on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0868](0868-ai-first-tooling-after-rust-removal.md), [ADR 0098](0098-pub-visibility.md) (visibility), Garlic agent server (`garlic --serve`)
- **Topic:** 20 — AI semantic API

## Context

Garlic speaks MCP through hand-written tool descriptions. Any AIL program with typed functions, docs and declared effects has the facts for the schema already.

## Decision

- `@tool` on a `pub fn` marks it as callable by an agent. Parameters and result must be JSON-representable (the `@derive` JSON types of ADR 0774).
- `ail serve --mcp [path]` runs the program as an MCP server on stdio. The tool list comes from the compiler: name, the `##` doc as description, a JSON Schema from the parameter types, and the function's effects in the description.
- A call decodes the JSON arguments, runs the function, and returns its JSON result. A failure value becomes an MCP error whose text is the error's message.
- A function whose effects exceed the capabilities the server was started with is not listed (ADR 0868 sandbox); the server prints why once at start.
- Planned rule keys: `tool.attr.*`, `tool.serve.*`.

Cost when unused (ADR 0868): none. The feature is a separate command or pass, so a program that does not use it gets no extra code, no runtime check and no slower build.

## Consequences

Garlic can drop its hand-written schemas later. Streaming results and long-running tools are out of scope for the first version.

## Rejected alternatives

- **A separate schema file:** drifts from the types; the types are the schema.
- **Every `pub fn` is a tool:** exposes helpers and hides intent; the attribute is explicit and queryable.
- **HTTP only:** MCP clients speak stdio first; HTTP can sit on `std.net` later.
