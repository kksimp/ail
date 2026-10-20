# 0283. `std.net`: a minimal HTTP/1.1 client, without TLS for now

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** Standard modules; effects and capabilities
- **Spec rules:** std.net.1–4, std.use.1

## Context

The `net` capability existed in the effects system (ADR 0163) with nothing
giving it. Agents fetch URLs: health checks, local services, APIs. The
bootstrap uses only the Rust standard library plus two UI crates
(ADR 0023, ADR 0177); TLS needs either a crate or a large amount of
cryptography.

## Options considered

### Option A — `get` and `post` over a small HTTP/1.1 client on `std::net`, no TLS yet (chosen)
- **Pros:** no new dependency; covers `http://` services, which is what
  local agent tooling mostly talks to; `net` is now real, so `--deny net`,
  package grants and `--deterministic` mean something.
- **Cons:** `https://` fails, with the kind `tls` and a message naming
  this ADR.

### Option B — add a TLS crate (e.g. rustls)
- **Pros:** `https` works.
- **Cons:** a large dependency tree, against ADR 0009/0023's bar; licence
  review is a designer decision (AGENTS.md escalation rule 4).

### Option C — shell out to `curl` through `proc`
- **Cons:** hides network access under the `proc` capability, so
  `--deny net` would not stop it; not present everywhere.

### Option D — `file:` URLs read local files
- **Cons:** `net.get` would then read files under the `net` capability,
  around `--deny fs`. Rejected; `examples/fetch.ail` instead shows the
  offline `tls` failure and a local fallback through `fs`.

## Decision

`net.get(url)` and `net.post(url, body, content_type)` return
`Response { status, headers Map[str, str], body }` or fail with
`NetError { url, kind, message }`. Any status is a success. `get` follows up
to five redirects. Timeouts are 30 s. Both have effects `net` and `io`.

## Rationale

Criterion 2: an HTTP 404 is an answer, not a failure, so the caller decides
by `status`, as `proc.run` treats a nonzero exit (ADR 0160). Headers are a
map with lowercase names so lookup needs no case rule. A `message` field
makes the TLS limitation self-explaining at the failure site.

## Consequences

- TLS is follow-up work; it needs a designer decision on a crate or an
  in-house implementation (ADR 0009).
- No streaming, custom headers, methods other than `GET`/`POST`, or
  per-call timeouts yet.

## Rejected alternatives

- A TLS crate tonight: dependency and licence decisions are the designer's
  (Option B).
- `curl` via `proc`: bypasses the `net` capability (Option C).
- `file:` URLs: bypass `--deny fs` (Option D).
- Non-2xx statuses as failures: forces `else` handling for ordinary
  answers and loses the body.
