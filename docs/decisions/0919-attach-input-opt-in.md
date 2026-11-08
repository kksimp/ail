# 0919. `ail run --attach-input`: an opt-in input channel for attach

- **Status:** Accepted
- **Date:** 2026-10-26
- **Decided by:** designer (Kaleb, 2026-10-26: "Yes, opt in")
- **Topic:** 17 — Standard library (`std.attach`)
- **Spec rules:** attach.input.1-3 (new), attach.wire.4 and attach.cli.1-3 (amended)
- **Builds on:** [ADR 0884](0884-observe-field-and-attach-protocol.md)
- **Amended by:** [ADR 0986](0986-app-tools-and-attach-mcp.md) (the opt-in also gates `call`)

## Context

Attach (ADR 0884) lets an agent read a running `std.app` program: `observe`, `screen`, `heap`. It cannot act. To test a UI an agent must read the screen, but cannot click a button or type into a field, so it cannot drive a flow end to end.

## Decision

A program started with `ail run --attach-input` also accepts one more request, `input`. It carries the fields of a `std.app` `Event` and the program handles it between its own events through `update`, as it would an event from the window. The reply is `{"v":1,"applied":KIND}`. Without the flag `input` is refused with a message that names the flag, and nothing changes.

- The flag sets `AIL_ATTACH_INPUT=1`; the attach wrapper reads it when the program starts, so a built executable can be driven with the variable and no rebuild or cache change.
- Only the kinds a person produces are accepted: `press`, `edit`, `submit`, `toggle`, `click`, `scroll`, `key`. `start`, `resize` and `external` are refused, so an agent cannot fake the host's own messages.
- The token and the 127.0.0.1-only listener (attach.wire.1, attach.wire.2) are unchanged and apply to `input`. The default `ail run` stays read-only.
- Client: `ail attach input kind=press id=pay`.

## Consequences

- An agent can click, type and submit, then confirm with `observe` or `screen`. The reply carries no state, which keeps the request cheap and one thing.
- With the flag on, anyone who can read the discovery file (mode 0600, the user) can act as the user in that program. The flag is the user's explicit choice per run.
- No cost when off: the same listener, one boolean test.

## Rejected alternatives

- **Always on.** It lets any local process holding the token act in every program run with `ail run`; the designer chose opt in.
- **A separate socket and token for input.** A second listener and file for no gain; the one token and the flag already gate it.
- **Reply with the new screen.** It spends tokens on every action; the agent asks for the part it needs.
- **Accept any event kind.** `resize` and `start` come from the host, and a forged `external` event could reach the attach handler itself.
- **A build-time flag only (`ail build --attach-input`).** It would need a rebuild and a second cache key for what one environment variable decides at start.
