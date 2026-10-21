# 0692. HTTP fuzzers, socket templates in the module harness, and a status-code fix

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 16 — Testing
- **Spec rules:** std.net.2, std.net.4, std.net.10, std.net.20-23, fuzz.mod.1-7, fuzz.policy.1
- **Builds on:** [ADR 0461](0461-module-fuzzers.md)

## Context

`http_server.ail` was meant to feed raw requests to `net.serve`. The harness runs every template with `--deterministic` and again with `--deny net`, and both refuse a socket before the program starts (`eff.cap.1`, exit 2, a rule ID on stderr). The template therefore never ran, and no property failed.

## Decision

1. A template header `# fuzz-net: yes` runs the program as it is (20 s limit) and skips the `--deny` tier and the repeat run. `http_server.ail` and the two new HTTP templates use it.
2. `http_client.ail` (input kind `httpresp`): a rogue listener answers a client `request` with a hostile raw response (odd status lines, header junk, bad `Content-Length`, malformed chunks, bogus `Content-Encoding`). The client returns a response or a `NetError`, with a status of three digits and lowercase header names.
3. `http_roundtrip.ail`: an echo server and the client; a POSTed body and a header come back unchanged.
4. `http_cookies.ail` (input kind `cookie`): hostile `Set-Cookie` lines against `std.net.jar`; the `Cookie` header has no control characters and scripts never see or set `HttpOnly` cookies.
5. **Finding:** the client took `HTTP/1.1 20 OK`, `+20` and similar as status codes (seed 22 of 300). A status code is exactly three digits, else kind `protocol` (std.net.4). Case: `fuzz_net_status_digits`.

Runs after the fix: 1,500 client responses, 300 server requests, 300 round trips and 2,000 cookie pairs, 0 failures.

## Rejected

- Letting `--deterministic` allow loopback sockets: it would change what deterministic means for programs.
- A separate Rust test for sockets: the template format already carries the properties.
