# Standard library: `std.oauth2`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/oauth2/oauth2.ail`, ADR 1267).

An OAuth 2.0 client ([RFC 6749](https://www.rfc-editor.org/rfc/rfc6749)) over the `std.net` HTTP client:
the authorization code grant with PKCE ([RFC 7636](https://www.rfc-editor.org/rfc/rfc7636)), client credentials, refresh
and the device authorization grant ([RFC 8628](https://www.rfc-editor.org/rfc/rfc8628)). The module is written in AIL
and is in scope without `use` (`std.use.4`); `oauth2.Client(...)` names it. A redirect listener, OpenID discovery, ID
token checks, DPoP and mutual-TLS client authentication are not in this module.

## The module

**std.oauth2.1** — `std.oauth2` provides `pkce`, `pkce_from`, `challenge_of`, `random_state`, `callback_params`,
`parse_token` and the types `Client`, `Token`, `Pkce`, `DeviceAuth`, `OAuthError` and `OAuthKind`. A `Client` has
`client_id` and `token_url` (required) and the optional `client_secret`, `auth_url`, `device_url`, `redirect_uri`,
`scope` (space-separated text), `secret_in_body` (false) and `timeout_ms` (30000). Its methods are `authorize_url`,
`exchange_code`, `client_credentials`, `refresh`, `fresh`, `device_start`, `device_poll` and `device_wait`. Every
call that sends a request has the effects of `net.request` (`eff.infer.1`). *(ADR 1267)*

```ail tool=run,FILE expect=app,rw rules=std.oauth2.1
fn main()
    c := oauth2.Client(client_id: "app", token_url: "https://idp.test/token", scope: "rw")
    print("{c.client_id},{c.scope}")
```

## PKCE and the authorization request

**std.oauth2.2** — `pkce() Pkce` returns a random verifier (43 characters from 32 random bytes of
`crypto.random_bytes`) and its challenge. `challenge_of(verifier str) str` is the S256 challenge: the unpadded
base64url of the SHA-256 of the verifier. `pkce_from(verifier str) Pkce ! OAuthError` checks a verifier of your own:
43 to 128 characters of `A-Z a-z 0-9 - . _ ~`, otherwise kind `BadConfig`. The plain method is not offered. *(ADR 1267)*

```ail tool=run,FILE expect=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM rules=std.oauth2.2
fn main() !
    p := oauth2.pkce_from("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk")?
    print(p.challenge)
```

**std.oauth2.3** — `c.authorize_url(state str, challenge str = "") str ! OAuthError` is `auth_url` followed by the query
`response_type=code`, `client_id`, `redirect_uri`, `scope`, `state` and, when `challenge` is not empty,
`code_challenge` and `code_challenge_method=S256`, each encoded as `url.encode_form` does; an empty value is left out,
and a query already on `auth_url` is kept (`&` joins). Without an `auth_url` it fails with kind `BadConfig`.
`random_state() str` is 22 characters of random base64url. `callback_params(link str) Map[str, str] ! OAuthError` is
the query of the redirect back to the app (`code`, `state`); when the query has `error` it fails with kind `Denied`,
the `code` set to that error and the `message` to its `error_description`. The caller compares `state`. *(ADR 1267)*

```ail tool=run,FILE expect=https://idp.test/auth?response_type=code&client_id=app&state=s1 rules=std.oauth2.3
fn main() !
    c := oauth2.Client(client_id: "app", token_url: "t", auth_url: "https://idp.test/auth")
    print(c.authorize_url("s1")?)
```

## Tokens

**std.oauth2.4** — A `Token` has `access_token`, `token_type`, and the optional `refresh_token`, `scope`, `id_token`
and `expires_at_ms` (Unix milliseconds, 0 when the server named no lifetime). `parse_token(body str, now_ms int) Token
! OAuthError` reads a token endpoint's answer, JSON or the form encoding some servers use; `expires_in` may be a
number or text and makes `expires_at_ms = now_ms + expires_in * 1000`; a missing `token_type` is `Bearer`. An answer
with `error` fails with kind `Denied`; one with no `access_token` fails with `BadResponse`. `t.expired_at(now_ms)` is
true from 30 seconds before `expires_at_ms`, and false for a token with no lifetime; `t.is_expired()` uses
`time.now_ms()`. `t.header()` is the `Authorization` value (`Bearer abc`); `t.has_scope(name)` tests the
space-separated `scope`. *(ADR 1267)*

```ail tool=run,FILE expect=a,3601000,true,false rules=std.oauth2.4
fn main() !
    t := oauth2.parse_token("\{\"access_token\":\"a\",\"expires_in\":3600}", 1000)?
    print("{t.access_token},{t.expires_at_ms},{t.expired_at(3601000 - 30000)},{t.expired_at(3601000 - 30001)}")
```

## Grants

**std.oauth2.5** — `c.exchange_code(code str, verifier str = "") Token ! OAuthError` posts `grant_type=authorization_code`,
`code`, `redirect_uri` and, when given, `code_verifier` to `token_url`. *(ADR 1267)*

```ail norun rules=std.oauth2.5
fn main() !
    c := oauth2.Client(
        client_id: "app",
        token_url: "https://idp.test/token",
        auth_url: "https://idp.test/auth",
        redirect_uri: "http://127.0.0.1:8080/cb",
    )
    p := oauth2.pkce()
    state := oauth2.random_state()
    print(c.authorize_url(state, challenge: p.challenge)?)
    # ... the user signs in and the browser lands on the redirect URL ...
    got := oauth2.callback_params("http://127.0.0.1:8080/cb?code=abc&state={state}")?
    t := c.exchange_code(got["code"], verifier: p.verifier)?
    print(t.header())
```

**std.oauth2.6** — `c.client_credentials() Token ! OAuthError` posts `grant_type=client_credentials` and the client's
`scope`; the client needs a `client_secret`, otherwise it fails with kind `BadConfig` before any request. *(ADR 1267)*

```ail tool=run,FILE expect=BadConfig rules=std.oauth2.6
fn main()
    c := oauth2.Client(client_id: "app", token_url: "https://idp.test/token")
    t := c.client_credentials() else err
        print(err.kind.name())
        return
    print(t.access_token)
```

**std.oauth2.7** — `c.refresh(t Token) Token ! OAuthError` posts `grant_type=refresh_token` and `t.refresh_token`; a token
with none fails with kind `BadConfig`. When the answer names no new `refresh_token` or `scope` the new token keeps the
old ones (RFC 6749 section 6). `c.fresh(t)` is `t` while it is not expired and `c.refresh(t)` otherwise. *(ADR 1267)*

```ail norun rules=std.oauth2.7
fn main() !
    c := oauth2.Client(client_id: "app", token_url: "https://idp.test/token")
    t := oauth2.Token(access_token: "old", token_type: "Bearer", refresh_token: "r1", expires_at_ms: 1)
    print(c.fresh(t)?.access_token)
```

**std.oauth2.8** — Every token request is a `POST` of `application/x-www-form-urlencoded` with `Accept: application/json`
and no redirects. A client with a `client_secret` sends it as `Authorization: Basic` over the form-encoded `client_id`
and secret (RFC 6749 section 2.3.1); with `secret_in_body` it sends `client_id` and `client_secret` in the form
instead, and a client with no secret sends `client_id` in the form. A 2xx answer is read by `parse_token`; any other
status with an `error` member fails with kind `Denied` (`status` set), and one without fails with `BadResponse`. A
request that gets no answer fails with kind `Network`, its `message` the `NetError`'s kind and text. A call needing a
URL the client lacks fails with `BadConfig`. *(ADR 1267)*

```ail tool=run,FILE expect=BadConfig rules=std.oauth2.8
fn main()
    c := oauth2.Client(client_id: "app", token_url: "")
    t := c.exchange_code("abc") else err
        print(err.kind.name())
        return
    print(t.access_token)
```

## Device flow

**std.oauth2.9** — `c.device_start() DeviceAuth ! OAuthError` posts the `scope` to `device_url`. A `DeviceAuth` has
`device_code`, `user_code`, `verification_uri` (`verification_url` is accepted), the optional
`verification_uri_complete`, `expires_at_ms` (from `expires_in`, default 600 seconds) and `interval_s` (default 5).
Show `user_code` and `verification_uri` to the user, then poll. *(ADR 1267)*

```ail norun rules=std.oauth2.9
fn main() !
    c := oauth2.Client(client_id: "app", token_url: "https://idp.test/token", device_url: "https://idp.test/device")
    d := c.device_start()?
    print("Open {d.verification_uri} and enter {d.user_code}")
    t := c.device_wait(d)?
    print(t.header())
```

**std.oauth2.10** — `c.device_poll(d) Token? ! OAuthError` asks the token endpoint once: the token when the user has
approved, `none` for `authorization_pending` and `slow_down`, kind `Expired` for `expired_token` and kind `Denied` for
any other error such as `access_denied`. `c.device_wait(d) Token ! OAuthError` polls until a token: it waits
`interval_s` seconds before each poll, adds 5 seconds to the wait after each `slow_down`, and fails with kind `Expired`
once `d.expires_at_ms` has passed. *(ADR 1267)*

```ail tool=run,FILE expect=Expired rules=std.oauth2.10
fn main()
    c := oauth2.Client(client_id: "app", token_url: "https://idp.test/token")
    d := oauth2.DeviceAuth(device_code: "d", user_code: "u", verification_uri: "v", expires_at_ms: 0, interval_s: 0)
    t := c.device_wait(d) else err
        print(err.kind.name())
        return
    print(t.access_token)
```

## Errors

**std.oauth2.11** — `OAuthError` is an error with `kind OAuthKind`, `code str`, `message str` and `status int`; its text is
`message`. `code` is the server's `error` (`invalid_grant`, `access_denied`, ...) or "" and `status` the HTTP status or
0. The kinds are `BadConfig` (a field the call needs is missing, or a PKCE verifier is malformed), `Network` (no
answer), `Denied` (an OAuth error answer), `BadResponse` (an answer that is not a token) and `Expired` (the device
code ran out). Match on `err.kind` and `err.code`, never on `message`. *(ADR 1267)*

```ail tool=run,FILE expect=Denied,access_denied rules=std.oauth2.11
fn main()
    m := oauth2.callback_params("http://h/cb?error=access_denied&state=x") else err
        print("{err.kind.name()},{err.code}")
        return
    print(m["code"])
```

**std.oauth2.12** — The module keeps no state: a `Client` and a `Token` are plain values, `Token` carries no timer, and
nothing refreshes in the background. The program stores a token (`std.kv`, a file) and calls `c.fresh(t)` before
use. The secret and tokens are never written to a log or an error message by this module. *(ADR 1267)*

```ail tool=run,FILE expect=Bearer rules=std.oauth2.12
fn main()
    t := oauth2.Token(access_token: "x", token_type: "bearer")
    print(t.header())
```
