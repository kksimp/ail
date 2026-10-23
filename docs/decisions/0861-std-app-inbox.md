# 0861. `std.app` inbox: tasks post events into the running app

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.14` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)

## Context

`app.run` blocks on the host's next line, so an app only changes when the user does something. Garlic needs the loop woken by an agent's socket connection, WebSocket messages, JS timers, watches and scheduled actions.

## Decision

1. `app.run_host` reads the host's lines in a task and puts them as events into a channel, together with events other tasks post. The loop takes the next event from that channel.
2. `struct Inbox` wraps the channel. `Inbox.post(ev)` queues an event; `app.external(source, data)` makes one with `kind` `external`, `id` the source and `text` the data.
3. `trait App` gets `attach(mut self, inbox Inbox)` (default: nothing), called once before the first `update`. An app starts its own tasks there (an `accept` loop, a timer) and gives them the inbox.
4. A host that exits posts `exited`, which fails `run` as before (`host_exited`).

Existing apps change nothing. Headless `script` has no inbox.

## Rejected alternatives

- **A host `tick` event every 50 ms.** Wakes the CPU 20 times a second while idle and adds up to 50 ms of latency.
- **A Swift-side external event channel.** Needs a host change per platform; the AIL side suffices.
- **A global inbox.** AIL has no module state, and a handle in the app is explicit.
