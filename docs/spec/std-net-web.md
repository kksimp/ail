# Standard library: `std.net.web`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1269).

What a small web service needs on top of the router of [std.net.http](std-net-http.md): cookies, sessions with a
pluggable store, CSRF tokens, CORS and `multipart/form-data` uploads with size limits. It is `use std.net.web`, used
as `web` (`std.use.4`). Every function works on `http.Req` and `net.Response`, so it serves the router and a plain
`http.serve` handler alike. A middleware takes a handler and returns a handler of the same type.

## Cookies

**std.net.web.1** — `cookies(req http.Req) Map[str, str]` is the cookies of a request, from every `Cookie` header:
pieces split on `;`, a value in double quotes loses them, a piece without `=` is skipped, and when a name repeats the
first wins (the browser sends the most specific path first). `cookie(req, name) str?` is one value. *(ADR 1269)*

**std.net.web.2** — `set_cookie(name str, value str, opts CookieOpts) str ! CookieError` is the text of a `Set-Cookie`
header. `CookieOpts` has `path` ("/"), `domain` (""), `max_age int?` (none), `secure` (false), `http_only` (true) and
`same_site` (`SameSite.Lax`; also `Strict` and `Open`, which writes `None`). The value is not escaped. It fails with a
`CookieError` whose `kind` is `BadName` (empty, or a separator or control character), `BadValue` (space, control,
`"`, `,`, `;`, `\`), `BadAttr` (`;` or a control in `path` or `domain`) or `BadPolicy` (`SameSite=None` without
`secure`; a `__Secure-` name without `secure`; a `__Host-` name without `secure`, with a `domain`, or with a `path`
other than `/`). *(ADR 1269)*

**std.net.web.3** — `clear_cookie(name, opts) str ! CookieError` is a `Set-Cookie` that makes the browser drop the
cookie (`Max-Age=0` and an `Expires` in 1970); `opts` must have the `path` and `domain` it was set with.
`with_cookie(resp net.Response, line str) net.Response` adds one `Set-Cookie` header to `resp`; the server sends each
as its own header (`std.net.17`). *(ADR 1269)*

**std.net.web.4** — Signed values. `sign(secret, name, value) str ! CookieError` is `value.MAC`, the MAC being the
HMAC-SHA-256 of `name=value` as lower-case hex, so a signed value does not work under another cookie name. `unsign(secret,
name, signed) str?` is the value when the MAC is right, compared in constant time (`std.crypto.1`), else `none`.
A secret under 16 bytes fails `sign` with `WeakSecret` and makes `unsign` give `none`. `new_secret() str` is 256 random
bits as 64 hex digits. `signed_cookie(req, secret, name) str?` and `set_signed_cookie(secret, name, value, opts)`
combine them with `cookie` and `set_cookie`. Signing proves a value was not changed; it does not hide it. *(ADR 1269)*

```ail rules=std.net.web.1,std.net.web.2,std.net.web.3,std.net.web.4
use std.net
use std.net.http
use std.net.web

fn main() !
    r := http.Req(
        method: "GET",
        path: "/",
        query: {},
        headers: {"cookie": "theme=dark; lang=\"en\""},
        body: "",
        peer: "",
        data: b"",
        server: -1,
        params: {},
    )
    print(web.cookies(r)["lang"])                                  # en
    print(web.set_cookie("sid", "abc", web.CookieOpts(secure: true))?)
    # sid=abc; Path=/; Secure; HttpOnly; SameSite=Lax
    resp := web.with_cookie(http.text(200, "ok"), web.clear_cookie("old", web.CookieOpts())?)
    print(resp.all_headers["set-cookie"].len())                    # 1
    secret := web.new_secret()
    signed := web.sign(secret, "user", "ann")?
    print(web.unsign(secret, "user", signed) else "forged")       # ann
    print(web.unsign(secret, "admin", signed) else "forged")      # forged
```

## Sessions

**std.net.web.5** — `trait Store` is where sessions live: `load(self, id str) Map[str, str]?` (none when unknown or
expired), `save(self, id str, data Map[str, str], expires_ms int)` and `remove(self, id str)`. A store is called from
many tasks at once. `memory_store(cap int) MemoryStore` is the one in the module: 16 shards, each behind a `Mutex`,
at most `cap` sessions (a new one past that drops expired ones, then those that expire first), lost when the program
ends; `purge()` drops the expired and returns how many, and `len()` counts. A `Session` is a handle (copies share its
data): `get(key) str?`, `set(key, value)`, `remove(key)`, `values()`, `id`, and `fresh` (true when this request began
it). Values are strings; keep structured data as JSON text. *(ADR 1269)*

**std.net.web.6** — `with_session(store any Store, opts SessionOpts, handler fn(http.Req, Session) net.Response)
fn(http.Req) net.Response` reads the cookie named `opts.name` ("sid"), loads the session, calls `handler`, then saves
the session with an expiry of `opts.ttl_ms` (one day) after now and sets the cookie (`HttpOnly`, `Path`, `Max-Age`,
`SameSite` and `Secure` from `opts`). A cookie that is not 64 hex digits, or names no stored session, begins a new
one. `SessionOpts` has `name`, `ttl_ms`, `path`, `domain`, `secure` and `same_site`. Each request that finds a live
session renews its expiry. *(ADR 1269)*

**std.net.web.7** — A new session is kept, and its cookie sent, only if the handler changed it (`set`, `remove`,
`regenerate`, or `csrf_token`); a visitor who stored nothing leaves no entry in the store. `s.destroy()` deletes the
session and sends a cookie that clears it. `s.regenerate()` keeps the data under a new random id (the old id is
removed), which is what to call at a login so an id planted before it is worthless. Both take effect when the
handler returns. *(ADR 1269)*

```ail rules=std.net.web.5,std.net.web.6,std.net.web.7
use std.net
use std.net.http
use std.net.web

fn visit(r http.Req, s web.Session) net.Response
    seen := s.get("n") else "0"
    n := (seen.to_int() else 0) + 1
    s.set("n", "{n}")
    http.text(200, "visit {n}")

fn main() !
    store := web.memory_store(10000)
    mut router := http.router()
    router.get("/", web.with_session(store, web.SessionOpts(), visit))
    srv := net.serve("127.0.0.1:0")?
    t := spawn serve_router(router, srv)
    url := "http://{srv.addr()}/"
    first := net.get(url)?
    print(first.body)                                             # visit 1
    sid := first.all_headers["set-cookie"][0].split(";")[0]
    print(net.get(url)?.body)                                     # visit 1 (no cookie, a new session)
    print(store.len())                                            # 2
    print(sid.len())                                              # 68
    srv.close()
    t.wait()?

fn serve_router(r http.Router, srv net.Server) ! Trapped
    r.run(srv)?
```

## CSRF

**std.net.web.8** — `csrf_token(s Session) str` is the session's token (64 hex digits), made and stored on first use.
`csrf_valid(req, s) bool` is true for `GET`, `HEAD` and `OPTIONS`, and for any other method when the header
`x-csrf-token`, or else the field `_csrf` of an urlencoded body, equals the token in constant time; a session without a
token is never valid. `csrf_guard(handler fn(http.Req, Session) net.Response)` answers `403 invalid CSRF token` when
`csrf_valid` is false, else calls `handler`; wrap what is passed to `with_session`. `same_origin(req) bool` is true
when `origin` (or else `referer`) names the `host` header, or neither is sent: an extra check, which a proxy that
rewrites `host` defeats. *(ADR 1269)*

```ail rules=std.net.web.8
use std.net
use std.net.http
use std.net.web

fn page(r http.Req, s web.Session) net.Response
    http.html(200, "<form method=post><input type=hidden name=_csrf value={web.csrf_token(s)}></form>")

fn save(r http.Req, s web.Session) net.Response
    http.text(200, "saved")

fn main()
    store := web.memory_store(1000)
    mut router := http.router()
    router.get("/form", web.with_session(store, web.SessionOpts(), page))
    router.post("/save", web.with_session(store, web.SessionOpts(), web.csrf_guard(save)))
    print("routes added")
```

## CORS

**std.net.web.9** — `cors(opts CorsOpts, handler fn(http.Req) net.Response) fn(http.Req) net.Response` adds the headers
of `opts` to the answers of `handler`. `CorsOpts` has `origins` (`["*"]`), `methods` (`GET HEAD POST PUT PATCH
DELETE`), `headers` (empty: allow what the preflight asks for), `expose`, `credentials` (false) and `max_age` (600).
An allowed origin gets `access-control-allow-origin`: `*` when any origin is allowed and `credentials` is false,
else the request's origin, with `vary: origin`. A preflight (`OPTIONS` with `access-control-request-method`) is
answered with `204` by the middleware itself, with `allow-methods`, `allow-headers` and `max-age`, and only when the
origin and the requested method are allowed; otherwise it gets `204` with no CORS headers and the browser blocks the
request. A request from an origin that is not allowed still reaches `handler`. `preflight(mut r http.Router, opts)`
adds the `OPTIONS /*path` route that answers every preflight, since a router matches by method. *(ADR 1269)*

```ail rules=std.net.web.9
use std.net
use std.net.http
use std.net.web

fn items(r http.Req) net.Response
    http.text(200, "[1, 2]")

fn main()
    opts := web.CorsOpts(origins: ["https://app.example"])
    mut router := http.router()
    web.preflight(mut router, opts)
    router.get("/items", web.cors(opts, items))
    print("ok")
```

## Uploads

**std.net.web.10** — `multipart(req http.Req, limits UploadLimits) Form ! FormError` reads a `multipart/form-data` body
(RFC 7578) from `req.data`. A `Form` has `fields Map[str, str]` (a repeated name keeps its last value) and `files
List[Part]` in the order sent; `form.file(name) Part?` and `form.files_named(name)`. A `Part` has `name`, `filename`,
`is_file`, `content_type` (`text/plain` when absent) and `data bytes`, and `text()`. A part with a `filename` is a file,
except one with an empty `filename` and no bytes (an empty file input), which is skipped. The `filename` is what the
client sent and must not be trusted: use `safe_filename`. The server holds the whole body and refuses one over 16 MiB
before the handler runs (`std.net.10`); there is no streaming to disk. *(ADR 1269)*

**std.net.web.11** — `UploadLimits` has `max_body` (10 MiB), `max_file` (8 MiB), `max_field` (64 KiB), `max_files`
(10) and `max_fields` (100). `multipart` fails with a `FormError` whose `kind` is `NotMultipart`, `NoBoundary` (no
`boundary` of 1 to 70 characters, quoted or not, in any case), `Malformed` (no opening boundary, a part without
header end, name or closing boundary, a part that is not `form-data`, headers over 8 KiB), `TooLarge`,
`TooManyParts`, `FieldTooLarge` or `FileTooLarge`. *(ADR 1269)*

**std.net.web.12** — `safe_filename(name str) str` keeps the last component after `/` or `\`, drops control
characters and `<>:"|?*`, strips leading dots and cuts to 255 characters; it gives `upload` when nothing is left.
The result never contains a separator, so `path.join(dir, safe_filename(x))` stays inside `dir`. *(ADR 1269)*

```ail rules=std.net.web.10,std.net.web.11,std.net.web.12
use std.net.http
use std.net.web

fn main() !
    body := "--B\r\nContent-Disposition: form-data; name=\"note\"\r\n\r\nhi\r\n--B\r\nContent-Disposition: form-data; name=\"f\"; filename=\"../x.txt\"\r\n\r\ndata\r\n--B--\r\n"
    r := http.Req(
        method: "POST",
        path: "/up",
        query: {},
        headers: {"content-type": "multipart/form-data; boundary=B"},
        body: body,
        peer: "",
        data: body.to_bytes(),
        server: -1,
        params: {},
    )
    form := web.multipart(r, web.UploadLimits(max_file: 1000))?
    print(form.fields["note"])                  # hi
    f := form.file("f") else
        return
    print(web.safe_filename(f.filename))        # x.txt
    print(f.data.len())                         # 4
    web.multipart(r, web.UploadLimits(max_file: 2)) else err
        print(err.kind.name())                  # FileTooLarge
```

## Rejected alternatives

- A router-wide middleware stack: `Router` has one `on_request` hook that sees the answer and cannot change it, and
  changing `http.ail` would collide with the other lanes. Wrapping a handler is explicit and local.
- Signed session cookies carrying the data: it makes every request larger and the data readable. A 256-bit random id
  with the data in a store is the common choice and works with any store.
- Signing the session id: the id is unguessable, so a MAC adds nothing; the cost is a secret to keep.
- Encrypted cookies: needs an AEAD with a key schedule and is easy to misuse; `sign` covers integrity.
- Key rotation inside `unsign`: a list of secrets is a small loop in the caller.
- Streaming uploads to disk: needs a server that hands over the connection; the server reads a whole body first.
