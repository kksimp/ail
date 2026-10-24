# 0869. HTTP proxies from the environment in the native `std.net` client

- **Status:** Accepted
- **Date:** 2026-10-23
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** standard library; 27 — Dependencies and tooling
- **Spec rules:** `std.net.30` (new)
- **Builds on:** [ADR 0369](0369-https-in-std-net.md), which left proxies as a follow-up

## Context

`std.net` connected directly and ignored `HTTP_PROXY` and `HTTPS_PROXY`
(ADR 0369). Behind a corporate or sandbox proxy no request could leave the
machine. Backlog lane 7 (networking) asks for proxy support.

## Decision

The native client reads the conventional environment variables. `HTTP_PROXY`
serves `http://` URLs and `HTTPS_PROXY` serves `https://` URLs, the lowercase
name first. `NO_PROXY` lists hosts that are reached directly: `*`, hosts, and
domain suffixes (a leading dot is optional), each with an optional `:port`.
Loopback names and addresses never use a proxy, so local test servers keep working.

- `http://` requests go to the proxy in absolute form (RFC 9112 section 3.2.2),
  with `Proxy-Authorization: Basic` when the proxy URL has user information.
- `https://` requests open a tunnel with `CONNECT host:port` (RFC 9110 section
  9.3.6), then run TLS to the origin through it. The proxy sees no plaintext.
- The pooled-connection key includes the proxy, so a pooled direct connection
  is never reused for a proxied request.
- A proxy URL must be `http://` (a bare `host:port` is read as such); anything
  else fails with `bad_url`. A refused `CONNECT` fails with `protocol`.
- Native only: the interpreter keeps connecting directly (ADR 0855, no double work).

Code: `compiler/rt/native/net_proxy.ail`, and `http_exchange_on` in `net_http.ail`.
Check: `tools/native-net/proxy-check.sh`.

## Rejected alternatives

- **A `proxy` field on `net.Options`:** changes a public API and the checker's
  struct for a rare need; the environment is how every other client is configured.
- **System proxy settings (macOS SCDynamicStore, Windows registry):** host-specific
  and large; they can come later if asked.
- **An `https://` proxy (TLS to the proxy itself):** rare, and the tunnel code
  would need a second TLS layer. It fails with a clear `bad_url` instead.
- **Reading the uppercase variable only:** many tools set the lowercase one,
  and it is the one curl prefers.

## Consequences

Not covered yet: a successful TLS tunnel in the test (it needs a CONNECT-capable
test proxy in front of a TLS origin), proxy authentication other than Basic,
and `NO_PROXY` CIDR ranges.
