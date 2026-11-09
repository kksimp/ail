# 0986. `@tool` methods: typed actions a running app offers agents, and `ail attach mcp`

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076); the feature is the designer's backlog item "agent-native app interfaces" (2026-10-25)
- **Amends:** [ADR 0884](0884-observe-field-and-attach-protocol.md) (two queries, a generated `call`), [ADR 0919](0919-attach-input-opt-in.md) (its opt-in now also covers `call`)
- **Narrows:** [ADR 0892](0892-tool-functions-mcp-server.md) (`@tool` on free functions stays Proposed; this ADR gives `@tool` its first meaning, on an app's methods)
- **Topic:** 20 — AI semantic API; std.app
- **Spec rules:** `attach.tool.1-5` (new); `attach.wire.4`, `attach.cli.1`, `attach.cli.3`, `test.attr.1` (amended)

## Context

An agent attached to a running `std.app` program can read its state (`observe`), read its screen (`screen`) and, with `--attach-input`, press and type (`input`). To add two teas to a cart it must find the field, edit it, find the button and press it, then read the result: several round trips that break when the layout changes. The program already knows the action, its arguments and their types. ADR 0879 point 3 says state changes go "through the program's own actions, such as a button or an MCP tool the program defines"; nothing let a program define one.

## Decision

1. `@tool` before a method `fn T.name(self | mut self, ...)` of a non-generic struct offers it to an attached agent. Parameters: `int`, `float`, `bool`, `str`, an enum without payloads. Result: none, or an `@observe`-able type, optionally fallible. The `##` doc is the description. Anything else is `attach.tool.2`.
2. The `attach.Observe` trait gains two defaulted methods, `tools(self) List[Tool]` and `call(mut self, name, args Json) str`; the built-in generator writes them from the `@tool` methods. The attach shim already adds `impl attach.Observe` to every app, so `ail run` needs nothing new.
3. Protocol: `tools` reads (token only) and lists each action with doc, parameter names and types, and result type. `call` acts, so it is refused unless the program was started with `--attach-input` (the ADR 0919 opt-in, unchanged). It runs between the app's events, like `input`. The reply is `{"v":1,"result":JSON}` or an error naming the bad argument, the unknown tool or the method's failure.
4. Arguments are read by type and accept text forms (`"2"` for an `int`), so `ail attach call add qty=2` works from a shell without typing hints.
5. `ail attach [--port N] mcp` serves the running program as an MCP server on stdio, the same wire as `ail mcp` (ADR 0918): `app_observe`, `app_screen`, `app_input` and one tool per `@tool` method with a JSON Schema from its parameter types. Each call is one attach request; the program is found afresh per call, so an agent harness can register the server before the app starts.

Cost when unused (ADR 0868): none. `@tool` methods are ordinary methods; `tools` and `call` are generated only into the `Observe` impl that attach adds.

Measured: driving "add 2 teas" took 4 requests by `input` (screen, edit, press, observe) and takes 1 `call`, whose reply carries the result.

## Consequences

- A `gfx.Game` window forwards its game's tools (`std.gfx.host.Loop`'s `Observe`), so a game's `@tool` methods work the same way.
- A hand-written `impl attach.Observe` writes `tools` and `call` itself or offers none.
- `List` parameters, optional parameters and structs as arguments are left out of version 1.

## Rejected alternatives

- **A separate `attach.Act` trait:** `serve.run` and `gfx.watch.run` would need a second bound, breaking every program that writes `impl attach.Observe` by hand; one trait keeps "what the program shows an agent" in one place.
- **`@tool` free functions here (ADR 0892 as proposed):** a free function cannot reach the running app's state; the method form is what an app needs. 0892's stdio server for free functions can still come later.
- **A new attribute `@action`:** two names for "an agent may call this"; `@tool` matches MCP's word, which is what agents read.
- **Calls without the `--attach-input` opt-in:** a second, ungated way to change a program, against ADR 0919.
- **Strict JSON types for arguments:** a shell client would need typing hints per argument.
