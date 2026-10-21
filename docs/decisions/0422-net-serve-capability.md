# 0422. `net.serve` is a capability on top of `net`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** standard library
- **Spec rules:** [std.net.14, eff.cap.1–5](../spec/17-std.md#network-stdnet)
- **Criterion that decided:** AI understanding (no ambiguity, good diagnostics), then tokens; speed is a tie

## Context

Starting a server changes what a program is: it can be reached. A
sandbox that allows a program to call an API may not allow it to listen.

## Options considered

### Option A — a second capability, `net.serve`, held with `net` (chosen)
- **Pros:** `--deny net.serve` leaves the client; `--deny net` and
  `--deterministic` still cover everything; no change to the denial
  machinery (names are text).
- **Cons:** one more name in manifests.

### Option B — split `net` into `net.connect` and `net.serve`
- **Pros:** symmetrical.
- **Cons:** changes the meaning of every existing `net` grant and
  `--deny net` use (ADR 0165, 0232); breaks lockfiles' `caps`.

### Option C — no new capability
- **Cons:** a dependency granted `net` could open a listening port.

## Decision

Option A. `net.serve` and `net.listen` have effects `net`, `net.serve`,
`io`. The methods of `Server`, `Exchange`, `Listener` and `Conn` carry `net`
and `io` only: obtaining a `Server` needs the call that has the capability, and
a function handed a `Server` is using what its caller was allowed.

## Consequences

- `CAPABILITIES` gains `net.serve`; `all` includes it.
- `ail effects` prints it for serving functions only.

## Rejected alternatives

- Splitting `net` into `net.connect` and `net.serve` (option B): rejected because it changes the meaning of every existing `net` grant and `--deny net` use (ADRs 0165, 0232), and it breaks lockfiles' `caps`.
- No new capability (option C): rejected because a dependency granted `net` could open a listening port.
