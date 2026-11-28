# 1288. `std.db.redis`: a Redis client in AIL

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.redis.1 to std.redis.13 in [std-redis.md](../spec/std-redis.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 0360](0360-in-house-tls-for-std-net.md) (TLS client), [ADR 1222](1222-std-net-websocket.md) (the model for a protocol module over `Conn`)


## Context

Programs that need a cache, a queue or a rate limiter reach for Redis, and `std.db` and `std.kv` do not talk to it.
An agent without a client writes RESP by hand over `net.connect`, and gets the framing, the null forms and the
pipelining wrong. The std plan lists it as a pure AIL module over `Conn`.

## Decision

- A module in AIL, `std/db/redis.ail`, used with `use std.db.redis` as `redis` (always written, like
  `std.net.websocket`). Nothing is added to the compiler or the runtime beyond the module's registration.
- The codec is public and pure: `encode`, `encode_bytes`, `decode` (returning how many bytes a reply took, 0 when
  incomplete) and a `Reply` enum for RESP2 and RESP3. This makes it testable with goldens and usable for a server
  or a proxy.
- `connect(url)` takes `redis://[user:password@]host[:port][/db][?protocol=3]` or `rediss://` (TLS through
  `net.connect_tls`), and does `AUTH`, `SELECT` or `HELLO 3` itself. RESP2 is the default because every server
  speaks it; RESP3 is a URL option.
- Typed helpers for strings, keys and expiry, hashes, lists, sets and sorted sets return plain AIL types (`str?` for a
  missing key, `Map[str, str]` for a hash). `command` and `command_bytes` return a `Reply` for everything else.
- An error reply is a failure (`RedisError`, kind `Server`, or `Auth`); inside a pipeline it is a `Reply.Failure`
  value, so one bad command does not hide the other replies.
- `Pipeline` queues encoded commands; `run` writes them once and reads one reply each; `transaction` wraps them in
  `MULTI` and `EXEC`.
- Pub/sub is a connection mode: `subscribe` and `next_message`, with messages that arrive during confirmation kept
  in order. A subscribed connection refuses commands with a distinct kind, so the mistake is a clear failure.
- `Pool` is a `Channel[Client]` filled at creation: `get` waits for a free connection, `put` gives it back and
  replaces a broken one.
- Tests are a native program that plays a scripted server over loopback and compares every request byte for byte;
  no Redis runs in the gate.

## Rejected alternatives

- **A runtime built-in.** No OS call is needed; the module is `Conn` reads and writes, and AIL code is easier to
  check and change.
- **Callbacks for pub/sub.** A `next_message` loop is plain and works with colorless tasks and cancellation.
- **Blocking list commands, `WATCH` and scripts as typed helpers.** They hold a connection past its timeout or need
  state across calls; `command` reaches them. They can be added without changing these signatures.
- **A `Reply` that decodes into `Json`.** It would lose the difference between bulk strings and bytes, integers and
  text, and the RESP3 types.
- **RESP3 by default.** A server older than Redis 6 answers `HELLO` with an error; the URL option is explicit.
- **Cluster and sentinel.** Both need topology and redirect handling; out of scope for a first client.

## Consequences

Windows and Linux use the same code over `Conn`. The gate test plays a scripted server that holds the requests the
helpers must send, so a wrong argument order shows as a mismatch. `tools/native-net/redis-check.sh` (not in the gate)
starts a real `redis-server` with a password and runs `redis_live.ail` against it: all typed helpers, a 1.5 MB value,
a 2000-command pipeline, transactions, pub/sub with a pattern, a pool shared by 8 tasks, RESP3 and auth failures. It
was green on Redis 7.0.
