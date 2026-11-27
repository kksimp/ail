# 1267. `std.oauth2`: an OAuth 2.0 client written in AIL

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the integrator assigns the number
- **Topic:** Standard library, networking
- **Spec rules:** [std.oauth2.1-12](../spec/std-oauth2.md)

## Context

Programs that call an API on a user's behalf need the OAuth 2.0 grants. Writing them by hand takes a PKCE
hash, a form body, a Basic header and a polling loop, and each is easy to get slightly wrong. The pieces exist:
`std.net` sends the requests, `std.url` encodes forms, `std.hash` and `std.crypto` give SHA-256 and random bytes.

## Decision

A module `std.oauth2` in AIL (`std/oauth2/oauth2.ail`, used as `oauth2`):

- A plain `Client` value (URLs, id, secret, redirect, scope); no hidden state, no background refresh. A `Token`
  holds `expires_at_ms` and answers `expired_at(now_ms)`, with 30 seconds of skew.
- Grants: authorization code with S256 PKCE, client credentials, refresh (keeping the old refresh token and scope when
  the answer names none, RFC 6749 section 6), and the device grant (RFC 8628) as `device_poll` (one step) and
  `device_wait` (loop with `slow_down`).
- Client authentication by Basic header by default (RFC 6749 section 2.3.1), or in the form with `secret_in_body`.
- Answers are read as JSON or as the form encoding some servers use. Failures are one `OAuthError` with an enum
  `kind` (`type.enum.7`) and the server's `code`.
- `parse_token`, `challenge_of`, `callback_params` are public so the pure parts are testable and usable with another
  HTTP client.

## Rejected alternatives

- **A redirect listener in the module.** It needs a browser and a port and belongs to the app; `std.net.http` already
  gives one in a few lines.
- **A token store and automatic refresh.** Hidden state and a background task; a program calls `c.fresh(t)` itself
  and keeps the token where it wants.
- **The `plain` PKCE method.** RFC 7636 says S256 whenever the client can; every current server supports it.
- **`scope` as `List[str]`.** The wire form is one space-separated string; a list adds a join and split at each edge
  for no gain.
- **OpenID discovery and ID token validation.** A separate module: it needs JWT and key sets.
- **A new native built-in.** Nothing here needs the runtime; AIL is enough.

## Consequences

The module is as fast as `std.net.request`. The tests run a token endpoint in the same program, so they need no
network. Tokens are returned to the caller and never logged by the module.
