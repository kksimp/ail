# 0638. A cookie jar in the standard library (`std.net.jar`)

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.net.20](../spec/17-std.md), std.net.21
- **Builds on:** [ADR 0637](0637-http-persistent-connections-and-timeouts.md); the Garlic browser (PR 227)

## Context

Agents that log in, and the Garlic browser, need cookies carried across
requests and redirects. Garlic's lane wrote a jar of its own
(`tools/garlic/jsdom/cookies.ail`, PR #227) with no paths, `Secure`,
`HttpOnly`, or `Max-Age` precedence, kept by host name alone.

## Decision

A jar in AIL, `bootstrap/src/std/net/jar.ail` (`use std.net.jar`), with the
RFC 6265 rules that decide where a cookie goes: domain matching (host-only
without `Domain=`), path matching with the directory default, `Secure` for
`https` only, `Max-Age` over `Expires` (the three common date layouts),
replacement by name, domain and path, and ordering by path length then age.
`Jar.request` and `Jar.get` wrap `net.request`, follow redirects themselves,
and send and store cookies on every hop. `for_script` and
`set_from_script` give `document.cookie` its view (no `HttpOnly`).

Pure AIL, so it runs in the interpreter and in native code alike, and has no
Rust surface to port to the self-hosted checker.

**Which one Garlic uses:** this one. Its `for_url(url)` map is
`Jar.for_script`, its `header(url)` is `Jar.header`, and its `set(url, line,
now_ms)` is `Jar.set_cookie(url, line)` (the jar reads the clock itself). The
Garlic cookie lane can switch after both land and delete `cookies.ail`.

## Not done

- A public suffix list: a response from `a.com` may set `Domain=com`.
- `SameSite`: a jar has no notion of a site or a top-level navigation.
- Per-host and total size limits (RFC 6265 recommends 50 and 3000).

## Rejected alternatives

- **Cookie state inside `net.request` (Rust).** It would hide state in a
  process-wide table and need the self-hosted checker ported; a jar the
  program holds is explicit and testable.
- **Keeping two jars, one for Garlic.** Two implementations of the matching
  rules will disagree; that is the bug class a shared library removes.
- **A `Jar` field in `net.Options`.** The Rust side would have to read an AIL
  struct, and redirects need the jar's cookies per hop.
