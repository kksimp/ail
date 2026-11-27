# 1271. `std.db.postgres`: a PostgreSQL client in AIL

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [std.postgres.1–14](../spec/std-postgres.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 0959](0959-std-crypto.md) (`std.crypto`), [ADR 1207](1207-std-db.md) (`std.db`)

## Context

`std.db` is an in-process engine. Programs that talk to a real database server need PostgreSQL first: it is the
server agents and services meet most. The std gap plan (2026-11-26) lists a database client among the missing
modules.

## Decision

- A module written in AIL, `std/db/postgres.ail`, used with `use std.db.postgres` (the `use` is always written,
  like `std.net.websocket`). It needs nothing new in the compiler or runtime: `net.Conn`, `crypto.pbkdf2_sha256`,
  `hash.hmac_sha256`, `hash.md5_legacy`, `crypto.random_bytes` and the `bytes` readers and pushes.
- Protocol 3.0 with trust, cleartext, md5 and SCRAM-SHA-256 logins. Parameters go through the extended protocol
  (Parse, Bind, Describe, Execute, Sync), so a value is never part of the SQL text. Parameters are sent as text with
  no declared type, and the server infers the type.
- The API follows `std.db`: `Value` (plus `Bool`), `Row` with `int`, `str`, `float`, `bytes`, `bool`,
  `query`, `exec`, `begin`, `commit`, `rollback`. `batch` runs a script through the simple protocol.
- The message encoders and decoders are public, pure functions, so a protocol exchange is tested with goldens and no
  server (suite `std_postgres`). A run against a live server is outside the gate.
- A small pool: `get` never waits (kind `Usage` when all are out), `put` keeps only an idle connection.

## Rejected alternatives

- **libpq through `ffi`:** a C dependency, and no use in the kernel or WASM targets.
- **Binary result format:** each type needs its own decoder; text form keeps `numeric`, dates and arrays exact.
- **Named prepared statements and a cache:** needs a per-connection cache in a pool; deferred until measured.
- **A blocking pool:** a task could deadlock on its own connections.
- **TLS in this change:** the in-band SSLRequest upgrade needed `Conn.start_tls`; it is added later in the same
  stack (std.postgres.15, `TBD-std-db-tls.md`).

## Consequences

Passwords go over the wire unencrypted on a network without a TLS tunnel (md5 and SCRAM do not send the password
itself, but the data is open). The spec says so. The ADR number and the index row come from the integrator.
