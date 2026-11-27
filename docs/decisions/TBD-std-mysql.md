# TBD. `std.db.mysql`: a MySQL and MariaDB client in AIL

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** [std.mysql.1–14](../spec/std-mysql.md)
- **Extends:** [ADR 0421](0421-raw-tcp-and-tls-conn.md) (raw `Conn`), [ADR 0959](0959-std-crypto.md) (`std.crypto`, `std.hash`), [ADR 1207](1207-std-db.md) (`std.db`)

## Context

`std.db` is an in-process engine, and the PostgreSQL client covers one server family. MySQL and MariaDB are the
other servers programs meet most. The std gap plan (2026-11-26) lists a database client among the missing modules.

## Decision

- A module written in AIL, `std/db/mysql.ail`, used with `use std.db.mysql` (the `use` is always written, like
  `std.net.websocket`). It needs nothing new in the compiler or runtime: `net.Conn`, `hash.sha1_legacy`,
  `hash.sha256` and the `bytes` readers and pushes.
- Protocol 10 handshake and protocol 41 packets, without CLIENT_DEPRECATE_EOF (every server supports the EOF form).
  Logins: `mysql_native_password` and the fast path of `caching_sha2_password`; an authentication switch is
  followed.
- The API follows `std.db` and the PostgreSQL client: `Value` (with `Bool`), `Row` with `int`, `str`, `float`,
  `bytes`, `bool`, `query`, `exec`, `batch`, `begin`, `commit`, `rollback`, a small pool.
- With parameters, `query` and `exec` prepare the statement, run it in the binary protocol and close it, so a value
  is never part of the SQL text. Without parameters they use the text protocol in one round trip. Both give the
  same `Value` for a type. Explicit `prepare`, `query_stmt`, `exec_stmt` and `close_stmt` reuse a statement.
- Dates, times, DECIMAL and JSON stay `Text` as MySQL prints them, so no precision is lost.
- The packet encoders and decoders are public, pure functions, so a protocol exchange is tested with goldens and no
  server (suite `std_mysql`). A run against a live server is outside the gate.
- A pool as in the PostgreSQL client: `get` never waits (kind `Usage` when all are out), `put` keeps only an idle
  connection with no open transaction.

## Rejected alternatives

- **libmysqlclient through `ffi`:** a C dependency, GPL-licensed, and no use in the kernel or WASM targets.
- **Always preparing, even without parameters:** one more round trip, and some statements cannot be prepared.
- **A cache of prepared statements per connection:** needs invalidation on reconnect and pool hand-back; deferred
  until measured. `prepare` gives the manual form.
- **The full `caching_sha2_password` exchange without TLS:** it needs RSA-OAEP, which `std.crypto` lacks. The full
  exchange over TLS is in `TBD-std-db-tls.md` (std.mysql.16).
- **CLIENT_DEPRECATE_EOF:** shorter result sets, but MariaDB and MySQL differ in the details; the plain form is
  the one every version speaks.
- **A blocking pool:** a task could deadlock on its own connections.

## Consequences

Without TLS (`TBD-std-db-tls.md`) the data crosses the network in the clear (the password does not: both logins send a salted
hash). The spec says so. `mysql_native_password` is off by default in MySQL 8.4 and later, and `caching_sha2_password`
needs a cached user, so a fresh MySQL 8.4 server may need one login from another client or a user set to
`mysql_native_password`. The ADR number and the index row come from the integrator.
