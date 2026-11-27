# TBD. TLS for `std.db.postgres` and `std.db.mysql`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 13 — Standard library
- **Spec rules:** std.postgres.15 ([std-postgres.md](../spec/std-postgres.md)), std.mysql.15 and 16 ([std-mysql.md](../spec/std-mysql.md))
- **Extends:** `TBD-std-postgres.md`, `TBD-std-mysql.md`, `std.net.42` (`Conn.start_tls`, `TBD-std-net-smtp.md`)

## Context

Both database clients shipped without TLS. A server on another host, and every managed database, needs it, and
MySQL 8.4 turns `mysql_native_password` off, so `caching_sha2_password` needs its full exchange, which needs TLS.
`Conn.start_tls` (the SMTP change) gives the in-band upgrade both protocols use.

## Decision

- `Config.tls` of type `Tls` (`Disable`, `Prefer`, `Require`), read from `?sslmode=` (PostgreSQL names) or
  `?ssl-mode=` (MySQL names). The default is `Prefer`: use TLS when the server offers it.
- PostgreSQL: SSLRequest (`00 00 00 08 04 d2 16 2f`), one answer byte (`S` upgrade, `N` no TLS), then `start_tls`.
- MySQL: when the greeting has `CLIENT_SSL`, the 32-byte SSL request as packet 1, `start_tls`, the login as packet 2
  with `CLIENT_SSL` set.
- Certificates and host names are always verified (system roots or `SSL_CERT_FILE`). There is no mode that skips it,
  and a failed handshake is never retried in the clear. Error kind `Tls` on both.
- MySQL's full `caching_sha2_password` exchange: over TLS the password is sent as a NUL-ended packet after `01 04`.
- The wire pieces (`encode_ssl_request`, `encode_login_tls`, `tls_from_text`) are pure and tested with goldens. The
  TLS exchanges have an interop job, `tools/interop/db-tls.sh`, against real PostgreSQL, MySQL and MariaDB servers
  in containers on a Linux host. It is not part of the gate.

## Rejected alternatives

- **Default `Disable`:** a program that forgets to ask sends its data in the clear.
- **Default `Require`:** breaks local servers without TLS; a developer would have to set a mode to run a test.
- **`Prefer` that retries in the clear when verification fails (libpq's `prefer`):** hides a man in the middle.
- **A `skip verification` mode:** TLS then protects nothing against an active attacker. A private CA goes in
  `SSL_CERT_FILE`.
- **The RSA-OAEP path for `caching_sha2_password` without TLS:** needs RSA encryption in `std.crypto`; TLS is the
  better answer.

## Consequences

`Prefer` can be stripped by an active attacker (the server's `N`, or a greeting without the capability);
`Require` is the setting for that case, and the spec says so. A self-signed MySQL 8 server (its auto-generated
certificate has no host name entry) fails the check under the default: run it with a certificate for its name, or
`ssl-mode=disable` on a trusted network. The ADR number and the index row come from the integrator.
