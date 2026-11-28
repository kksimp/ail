# 1313. SMTP, SSE, S3 and OAuth 2 checked against real implementations; two runtime hangs fixed

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer's rule that the std must work, not only compile
- **Topic:** 17 — Standard library; native runtime
- **Spec rules:** [std.net.12](../spec/17-std.md), [std-net-smtp](../spec/std-net-smtp.md), [std-sse](../spec/std-sse.md)

## Context

The state review listed SMTP, SSE, S3 and OAuth 2 as modules with no interop check. Running them against real
servers found two runtime hangs: the first connect after a large allocation could hang, because a recycled block
left the socket table's lock word held, and an `accept` blocked on a listener did not end when another task closed
the listener (closing a descriptor wakes no `poll` on Linux).

## Decision

- `tests/interop/` gains `smtp`, `sse`, `s3` and `oauth2` (aiosmtpd; curl, requests, sseclient and the standard's
  parser; SeaweedFS with boto3 and curl; Ory Hydra and Authlib's device flow), driven by `net_servers.py`. `s3` and
  `oauth2` need docker and run only when named. 162 of 162 checks pass.
- The socket table's lock word is zeroed when its block is allocated, and a blocked `accept` rechecks its listener
  at least every quarter second and ends with `Closed` once it is closed (`std.net.12`).
- Fixes the checks found: the SSE client ends a line at a lone CR, ignores an id that holds NUL and reports a
  non-HTTP status line as `Protocol`; SMTP keeps a text part's final line break inside multipart. The
  `std_oauth2_flow` golden takes `header()` as `Bearer TOKEN`, as Hydra answers.

## Rejected alternatives

- **Mock servers only.** They agree with the code that wrote them; the hangs and the SSE edge cases showed only
  against real implementations.
