# 0188. Channels: `Channel[T](capacity)`, `send`, `recv() T?`, `close`

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Concurrency ([status.md](../status.md) topic 11)
- **Spec rules:** conc.share.3, conc.share.4

## Context

ADR 0100 decided bounded channels and copying, not the operations.

## Options considered

### Option A — `send(v)`, `recv() T?` (`none` when closed and empty), `close()`; capacity at least 1 (chosen)
- **Pros:** `v := ch.recv() else break` ends a consumer loop in one line with
  existing syntax; closing is how a producer says "done".
- **Cons:** no rendezvous (capacity 0) channels yet.

### Option B — `recv() T ! Closed`
- **Cons:** an end of stream isn't an error (ADR 0051 keeps absence and
  failure apart); more tokens at every use.

### Option C — capacity 0 means unbuffered rendezvous
- **Cons:** another blocking mode to explain; deferred until needed.

## Decision

Option A. `send` on a closed channel traps; closing twice traps; a
capacity below 1 traps. A channel value is a handle: copies are the same
channel.

## Rationale

Tokens and AI understanding (ADR 0076, criteria 1-2): the receive loop is
the most common use, and `else break` is the shortest unambiguous form.
Traps on misuse catch bugs where they happen.

## Consequences

- `select` over several channels stays an open design topic.

## Rejected alternatives

- `T ! Closed`: confuses absence with failure and costs tokens.
- Rendezvous channels now: no use case yet.
