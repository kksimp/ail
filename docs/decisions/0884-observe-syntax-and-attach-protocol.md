# 0884. `@observe` is an accessor attribute; attach speaks JSON lines on loopback

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076; amends [ADR 0879](0879-observe-and-live-attach.md) and changes none of its points: read-only, local only with a 0600 token file, on in `ail run`, opt-in `--attach` for `ail build`, zero cost when off)
- **Topic:** 20 — AI semantic API; std.app
- **Spec rules:** `attach.observe.1-4` and `attach.wire.1-6` (named below; they enter the spec with the code that cites them)

## Context

ADR 0879 fixes what `@observe` and attach do. It leaves the concrete syntax and wire format open. Constraints: attributes in the grammar apply to functions only (`gram.attr.1-2`); the front end is shared by every lane (`parse.ail`, the formatter, `ail api`); Garlic already runs a token-guarded loopback server on the `std.app` inbox (ADR 0861).

## Options considered

### Option A — `@observe` on an accessor function (chosen)
`@observe fn Cart.total(self) int` returns the value an agent may read. The attribute reuses `gram.attr.1-2`.
- **Pros:** no grammar, formatter or `ail api` change; read-only by type (`self` is not `mut`); the value is checked like any code and can be derived (a count, a sum); the observed name and type are the function's.
- **Cons:** about 3 lines per plain field rather than 1.

### Option B — `@observe` before a field (`@observe total int`)
- **Pros:** shortest for plain fields.
- **Cons:** new field-attribute grammar in `parse.ail`, the formatter, `ail api` and the checker (hot files every lane touches); a field can't be derived or reformatted for agents.

### Option C — a struct-level list (`@observe("a, b")`)
- **Cons:** names fields in a string the checker can't see; drifts on rename.

## Decision

1. **Syntax.** `@observe fn T.name(self) V` marks an observed value. `V` is `int`, `float`, `bool`, `str`, an enum without payload, or a list of those. The function takes `self` only and has no effects (`attach.observe.1`). Anything else is an error with a fix (`attach.observe.2`). Names are unique per type (`attach.observe.3`).
2. **Registry.** With attach on, the compiler appends to each `impl App for T` that has observed functions a method `observed(self) List[attach.Observed]` (name, type, value as JSON text). `std.app.App` has a default `observed` returning `[]`. With attach off the attribute is checked and nothing is generated (`attach.observe.4`).
3. **Attach on.** `ail run` compiles with attach on unless `--no-attach`; `ail build` only with `--attach`. Attach off links no listener, no `observed` methods and no thread (ADR 0879 item 4).
4. **Transport.** One thread listens on `127.0.0.1` on a free port. The address and a 24-byte random token go to `$TMPDIR/ail-attach-<pid>.json`, created mode 0600 and removed at exit (`attach.wire.1`). Every request carries the token (`attach.wire.2`).
5. **Protocol.** One JSON object per line in, one per line out, version 1 (`attach.wire.3`). Queries: `{"q":"observe","names":[…],"since":N}`, `{"q":"screen",…}` (the `std.screen` queries with `budget`), `{"q":"heap"}`. There is no query that writes (`attach.wire.4`). `observe` answers `{"v":1,"rev":N,"values":{name:value}}`; with `since`, only values that changed after revision N (a watch is polling by `since`, `attach.wire.5`).
6. **Threading.** The listener never reads program state itself. It posts a request on the app's inbox and the app answers between events, so safe code stays data-race-free (ADR 0025, `attach.wire.6`).

## Rationale

Tokens first (ADR 0076): Option B saves about 2 lines per plain field but costs a grammar change in three shared tools and cannot express derived values; the saving is under the 2% threshold on representative programs. AI understanding: an accessor is ordinary checked code, and read-only follows from the type.

## Consequences

Implementation is staged: spec and this ADR; the `std.attach` module with the server and protocol; the attribute checks and `observed` generation; the `ail run` and `--attach` flags. Plain (non-`std.app`) programs get no attach in this ADR.
