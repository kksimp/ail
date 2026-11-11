# 0884. `@observe` marks a struct field; attach speaks JSON lines on loopback

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076; amends [ADR 0879](0879-observe-and-live-attach.md) and changes none of its points: read-only, local only with a 0600 token file, on in `ail run`, opt-in `--attach` for `ail build`, zero cost when off)
- **Amended by:** [ADR 0979](0979-gfx-windows-publish-attach.md) (the shim also covers `std.gfx` windows), [ADR 0986](0986-app-tools-and-attach-mcp.md) (`tools` and `call` queries for `@tool` methods), [ADR 1045](1045-attach-listener-on-windows.md) (the directory on Windows, and how the listener stops)
- **Topic:** 20 — AI semantic API; std.app
- **Spec rules:** `attach.observe.1-4` and `attach.wire.1-6` (named below; they enter the spec with the code that cites them)

## Context

ADR 0879 fixes what `@observe` and attach do, and gives the form `@observe cart_total int`. It leaves the wire format and the generated code open. Constraints: attributes in the grammar apply to functions only today (`gram.attr.1-2`), so the field form needs a small addition to the struct grammar; Garlic already runs a token-guarded loopback server on the `std.app` inbox (ADR 0861).

## Options considered

### Option A — `@observe` before a field, as approved in ADR 0879 (chosen)
`@observe cart_total int` inside a `struct` body.
- **Pros:** shortest form; it is what the designer approved; the value is the field itself, so read-only and always current.
- **Cons:** a grammar addition (`ast.Field.observe`, `parse.ail`, the formatter, `ail api`); no computed values.

### Option B — `@observe` on an accessor function (`@observe fn Cart.total(self) int`)
- **Pros:** no grammar change; can derive values.
- **Cons:** not the approved form; three lines per plain field. Kept as an open question for computed values, for the designer.

## Decision

1. **Syntax.** In a struct body, `@observe` may precede a field: `@observe cart_total int`. The field's type is `int`, `float`, `bool`, `str`, an enum without payload, or a list of those (`attach.observe.1`). Any other type is an error with a fix (`attach.observe.2`). Only `pub` or private fields of a struct that has an `impl App` are observed by `std.app` (`attach.observe.3`).
2. **Registry.** `std.attach.Observe` has one method, `observed(self) List[attach.Observed]` (name, type, value as JSON text), whose default reports nothing. An `impl attach.Observe for T` with no methods gets `observed` from a built-in generator, like `ToJson` (ADR 0850), reporting each `@observe` field (`attach.observe.4`). With attach on, the compiler adds that empty `impl` for every `std.app` type that has observed fields; with attach off it adds nothing, and the attribute is only checked.
3. **Attach on.** `ail run` compiles with attach on unless `--no-attach`; `ail build` only with `--attach`. Attach off links no listener, no `observed` methods and no thread (ADR 0879 item 4).
4. **Transport.** One thread listens on `127.0.0.1` on a free port. The address and a 24-byte random token go to `$TMPDIR/ail-attach-<port>.json`, created mode 0600 and removed at exit (`attach.wire.1`). Every request carries the token (`attach.wire.2`).
5. **Protocol.** One JSON object per line in, one per line out, version 1 (`attach.wire.3`). Queries: `{"q":"observe","names":[…],"since":N}`, `{"q":"screen",…}` (the `std.screen` queries with `budget`), `{"q":"heap"}`. There is no query that writes (`attach.wire.4`). `observe` answers `{"v":1,"rev":N,"values":{name:value}}`; with `since`, only values that changed after revision N (a watch is polling by `since`, `attach.wire.5`).
6. **Threading.** The listener never reads program state itself. It posts a request on the app's inbox and the app answers between events, so safe code stays data-race-free (ADR 0025, `attach.wire.6`).

## Rationale

The designer approved the field form in ADR 0879, and it is also the cheapest in tokens. Computed values (a sum or count) are not covered; the accessor form (Option B) is the candidate and is [OQ-140](../open-questions.md), the designer's call.

## Consequences

Implementation is staged: spec and this ADR; the `std.attach` module with the server and protocol; the field attribute in the parser, formatter and checker, and `observed` generation; the `ail run` and `--attach` flags. Plain (non-`std.app`) programs get no attach in this ADR.

## Amendment 2026-10-24: how attach is switched on

`ail run` and `ail build --attach` rewrite the program's source text before lowering (`attach.run.1-3`): `app.run(` becomes `std.attach.serve.run(`, and the empty `impl attach.Observe` of item 2 is added for each `impl app.App for T`. Rejected: a hook in `std.app.run` (a cycle, `std.app` → `std.screen.view` → `std.app`, and a branch in every program), and an environment variable read at run time (the code would be linked in when off). A rewritten program that doesn't build falls back to the program as written, with a warning. Limits: only `app.run(` through an import named `app`; generic app types get no empty impl.
