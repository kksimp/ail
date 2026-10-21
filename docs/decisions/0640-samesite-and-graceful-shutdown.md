# 0640. SameSite in the cookie jar, and a graceful server shutdown

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 21 — Standard library architecture
- **Spec rules:** [std.net.23](../spec/17-std.md), std.net.24
- **Builds on:** [ADR 0638](0638-cookie-jar.md), [ADR 0639](0639-http-server-keep-alive.md)

## Context

A browser (Garlic) must not send `SameSite=Strict` cookies on a request that a
page on another site causes, or `Lax` ones on a subresource. A server needs to
stop without cutting off a request already arriving.

## Decision

1. **SameSite** in `std.net.jar`: the cookie records `lax` (the default, as
   browsers treat a missing value), `strict` or `none`. `none` without
   `Secure` is refused. `header_from(url, from, navigation)` applies the rule
   by the site of `from` against the site of `url`; the plain `header` is the
   same-site case, so existing callers are unchanged. A site is the last two
   host labels; there is no public suffix list yet, so `a.co.uk` and `b.co.uk`
   count as one site. That errs towards sending, never towards leaking a
   cookie to a different registrable domain.
2. **`srv.shutdown()`**: the listener is dropped after taking the connections
   already waiting at it; idle keep-alive connections are closed; requests in
   progress are answered with `Connection: close`; `accept` then fails with
   `closed` once none are left. `close()` is unchanged.

## Rejected alternatives

- **`SameSite` decided inside `net.request`.** The client cannot know the page
  that caused a request; the caller passes `from`.
- **Treating a missing `SameSite` as `none`.** The old behaviour, and the one
  every current browser has left.
- **A timeout argument on `shutdown`.** Requests are bound by the 10 s request
  limit already (`std.net.10`); a second timer adds nothing.
