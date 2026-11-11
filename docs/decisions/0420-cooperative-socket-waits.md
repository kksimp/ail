# 0420. Network waits are cooperative: non-blocking sockets, yield, back off

- **Status:** Accepted (option B in part by [1009](1009-readiness-io.md))
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** standard library
- **Spec rules:** [std.net.8](../spec/17-std.md#network-stdnet)
- **Criterion that decided:** AI understanding (no ambiguity, good diagnostics), then tokens; speed is a tie

## Context

Tasks run one at a time, each on its own thread, passing a turn
(ADR 0186). A task that blocks in a socket call holds the turn, so the
task it waits for never runs. The first test of a client and a server in
one program hung for this reason.

## Options considered

### Option A — non-blocking sockets; a wait yields to ready tasks, then sleeps 50 µs growing to 2 ms (chosen)
- **Pros:** no change to the scheduler's rules; works for TLS (the in-house
  stream is generic over its transport); cancellation and a timeout are
  checked in the same loop; deterministic apart from timing.
- **Cons:** polling: up to 2 ms of added latency and some CPU while
  idle; it never beats a readiness API.

### Option B — a readiness API (`epoll`, `kqueue`, `poll`)
- **Pros:** no polling, lower latency, scales to many connections.
- **Cons:** a platform layer per OS written without libc crates, three
  implementations to test in lockstep (ADR 0008); a refinement that can
  replace the sleep in `pause` without changing the API.

### Option C — release the turn around every blocking call
- **Pros:** simplest.
- **Cons:** two tasks would then touch `Rc` values at once; the scheduler's
  soundness argument depends on one runner.

### Option D — run network calls on helper threads
- **Pros:** real blocking.
- **Cons:** results must cross threads; the same `Rc` problem.

## Decision

Option A. Every wait loops: poll, give the turn to the next ready task
(`Yielder`, `interp/tasks.rs`), back off. Every client call uses the polling
stream, so a lone `net.get` also pays up to 2 ms of added latency per wait
(measured below); a readiness API (Option B) is the planned replacement for
the sleep and changes no API.

## Measurement

200 sequential `net.get` calls to a server task in the same program take 90 ms on Linux x86-64 (0.45 ms each, loopback), so the polling cost is small next to a real network round trip.

## Consequences

- `flush` from a task other than the main one no longer prints a blank
  line (a latent bug found here).
- `time.sleep_ms` in a task still holds the turn; only network waits yield.

## Rejected alternatives

- Releasing the turn around every blocking call (option C): rejected because two tasks would then touch `Rc` values at once, and the scheduler's soundness argument depends on one runner.
- Helper threads for network calls (option D): rejected because results must cross threads, which is the same `Rc` problem.
- A readiness API (`epoll`, `kqueue`, `poll`) (option B): not chosen now. It needs a platform layer per OS and three implementations to test in lockstep (ADR 0008). It is the planned replacement for the sleep and changes no API.
