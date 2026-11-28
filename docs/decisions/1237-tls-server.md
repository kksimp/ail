# 1237. A TLS 1.3 server: `net.serve_tls`, `net.listen_tls`, `http.serve_tls`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); shipping on unaudited in-house signing approved by the designer (2026-11-27, std wave 2)
- **Topic:** 13 — Standard library
- **Spec rules:** [std.net.46–49](../spec/17-std.md)
- **Extends:** [ADR 0360](0360-in-house-tls-for-std-net.md) and [ADR 0836](0836-native-tls.md) (the TLS client), [ADR 0419](0419-http-server-as-accept-and-respond.md) (the HTTP server), [ADR 1217](1217-http-router.md) (the router)

## Context

`std.net` had a TLS 1.3 client but no server ("There is no TLS server", std.net.13), so an AIL service needed a
proxy in front of it for `https`. The std gap plan (lane 15) asked for the server over the client's record layer,
with the signing of lane 14.

## Decision

- **API.** `net.serve_tls(addr, cert, key) Server` and `net.listen_tls(addr, cert, key) Listener` are `serve` and
  `listen` with a PEM certificate chain file and a PEM key file. The result types are the plain ones, so
  `Server.accept`, `Exchange.respond`, `Conn.read` and the rest, and `http.run` and `Router.run`, work unchanged.
  `http.serve_tls(addr, cert, key, handler)` and `Router.serve_tls(addr, cert, key)` complete the handler form.
  `net.listen_tls_alpn(addr, cert, key, protocols)` and `Conn.alpn()` let an HTTP/2 server (another lane) choose
  `h2`.
- **Files, not PEM text.** Every server in use reads files (`cert.pem`, `key.pem`); paths keep the call short and
  the messages can name the file. Load failures (unreadable, no certificate, encrypted key, wrong kind, a key that is
  not the certificate's) fail `serve_tls` itself with kind `tls`, before binding, so a misconfiguration is seen at
  start, not at the first client.
- **Protocol.** TLS 1.3 only, the client's three suites, X25519 with a HelloRetryRequest when the client offered
  X25519 without a share (OpenSSL with `-groups P-256:X25519`, some Java clients). Keys: ECDSA P-256 (lane 14's
  constant-time RFC 6979 signing) and RSA 2048-8192 with RSASSA-PSS, written here: CRT with Montgomery ladders whose
  multiplications and masked selects don't depend on the exponent bits, and every signature checked with the public
  key before it is sent (a faulty signature would leak a factor). No resumption, early data or client certificates.
- **Shape.** `rt/native/tls_server.ail` (handshake) and `tls_keys.ail` (keys and signing) beside the client; the
  record layer, key schedule, `tc_read` and `tc_write` are the client's. A server connection's state is the client
  layout with `TS_ROLE` 1, which also makes the shared messages say "the client" instead of "the server".
- **Steps.** On a `Server` the handshake moves one client flight per wake-up of `accept` (ClientHello, then Finished),
  and the connection is parked between them as a slow HTTP request is (std.net.10), so a slow or silent client holds
  up no other. On a `Listener`, `accept` runs the whole handshake (10 s a flight) before returning; a failed handshake
  is dropped and `accept` waits for the next client, so a port scanner can't end a server loop written with `?`.
- **Failures are told to the client.** The alert names the cause (`protocol_version`, `handshake_failure`,
  `no_application_protocol`...); a client that sends plain HTTP gets a plain-text `400` saying to use `https://`.

## Rejected alternatives

- **PEM text arguments** (`serve_tls(addr, cert_text, key_text)`): saves a file read in tests but makes the common
  case longer and loses the file name in messages. A program can still write the text to a file.
- **An options struct** (`net.TlsOptions { cert, key, alpn }`): one more type for three fields; `listen_tls_alpn` is
  the one variant needed now.
- **Failing `Listener.accept` on a bad handshake**, as the WebSocket server does for a bad upgrade: a scanner or a
  plain-HTTP client would end `l.accept()?` loops. The client gets the reason instead.
- **The whole handshake inside one `Server.accept` wake-up**: simpler, but a client that sends its ClientHello and
  stops holds that accepting task for the timeout.
- **RSA PKCS#1 v1.5 signatures**: TLS 1.3 forbids them in CertificateVerify.
- **Ed25519 certificates**: lane 14 can sign, but the client doesn't offer the scheme and few clients accept such
  certificates; a key file of that kind gets a message.
- **Serving `h2` from `serve_tls` now**: the HTTP server is HTTP/1.1; the HTTP/2 lane builds on `listen_tls_alpn`.

## Consequences

- A connection keeps about 170 KiB of TLS buffers after the handshake (the client's sizes); shrinking the
  post-handshake buffer is follow-up work.
- No `close_notify` is sent when a server closes a connection; responses are framed, so clients don't need it.
- RSA private-key operations are not blinded; the masked ladder removes the exponent-dependent branches but the
  Montgomery multiplication's final subtraction is still conditional. The spec says unaudited.
- Tests: `tests/native/run/net_tls_serve.ail` (both key kinds, three key formats, `listen_tls`, plain HTTP, load
  failures) and `tools/native-tls/server-check.sh` (OpenSSL with each suite, HelloRetryRequest, TLS 1.2 and PKCS#1
  refused, RSA-PSS with SHA-256 and SHA-384, curl).
