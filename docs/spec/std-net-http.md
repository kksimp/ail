# Standard library: `std.net.http` server

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in the bootstrap (ADRs
> 0754 and 0755); the router (`http.route.*`) is native only (ADR 1217). The
> server itself is `std.net.9`–`11`, `22` and `24`, and the handler form is
> `std.net.18`; this chapter adds what an agent needs to build a small
> service on top of them.

**http.serve.1** — `req.stop()` on the `net.Request` a handler received
stops the server that received it, as `srv.shutdown()` does
(`std.net.24`): the request under way is still answered, with
`Connection: close`, idle connections are dropped, and `http.serve` or
`http.run` returns once the other requests have been answered. `Request`
has a last field `server int`, the server's handle; a `Request` made by hand
should give `-1`, and `stop()` on it does nothing. *(ADR 0754)*

**http.serve.2** — `use std.net.http` also gives `serve_dir(addr str, root
str) ! net.NetError` and `run_dir(srv net.Server, root str) !`, the same as
`serve` and `run` with a built-in handler that answers `GET` and `HEAD` with
the file under `root` named by `Request.path`. A path ending at a directory
answers its `index.html`. The `content-type` comes from the extension (`html`,
`css`, `js`, `json`, `txt`, `md`, `xml`, `svg`, `png`, `jpg`, `gif`, `webp`,
`ico`, `pdf`, `wasm`), in any case, and is `application/octet-stream`
otherwise. A path with a `..` segment gets `403`, a missing file or an
unreadable one `404`, and any other method `405` with `Allow: GET, HEAD`. The path is used as sent: `%XX` is not decoded, and a
symbolic link inside `root` is followed. *(ADR 0755)*

**http.serve.3** — A response is always framed by `Content-Length`
(`std.net.11`); there is no chunked response. A client that keeps a
connection alive makes 100 sequential requests in under 0.5 s on Linux
x86-64 (about 3 ms each, measured by `http_serve_dir` in the bootstrap's
cases). *(ADR 0754)*

```ail rules=http.serve.1,http.serve.2,http.serve.3
use std.net
use std.net.http
use std.fs

fn hello(r net.Request) net.Response
    if r.path == "/stop"
        r.stop()
    net.Response(status: 200, headers: {}, body: "hi {r.path}")

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn http.run(srv, hello)
    print(net.get("http://{srv.addr()}/a")?.body)
    print(net.get("http://{srv.addr()}/stop")?.body)
    t.wait()?
    fs.make_dir("http_spec_dir.tmp")?
    fs.write_text("http_spec_dir.tmp/index.html", "<p>ok</p>")?
    d := net.serve("127.0.0.1:0")?
    w := spawn http.run_dir(d, "http_spec_dir.tmp")
    r := net.get("http://{d.addr()}/")?
    print("{r.status} {r.headers["content-type"]} {r.body}")
    print(net.get("http://{d.addr()}/../x")?.status)
    d.close()
    w.wait()?
    fs.remove("http_spec_dir.tmp/index.html")?
    fs.remove("http_spec_dir.tmp")?
```

## Router

**http.route.1** — `router() Router` makes a router with no routes.
`r.get(pattern, handler)`, `post`, `put`, `patch` and `delete` add a route
for that method, `r.any(pattern, handler)` one for every method, and
`r.route(method, pattern, handler)` one for `method` in any case (`"*"` is
every method); each needs `mut r`. A handler is `fn(http.Req)
net.Response`, a declared function or a literal (`|req| show(req, store)`,
`type.fn.4`). A pattern and a request path are split at `/` and their empty
segments dropped, so `//a/b/` is `/a/b`. A segment `:name` matches any one
segment and `*name` the rest of the path, none or more segments joined with
`/` (segments after it are ignored); others match only themselves. A
captured value has `%XX` decoded. The routes are tried in the order they
were added and the first whose pattern and method match answers; a `GET`
route answers `HEAD` too (the server sends no body, `std.net.11`).
*(ADR 1217)*

**http.route.2** — A handler gets `struct Req { method str, path str, query
Map[str, str], headers Map[str, str], body str, peer str, data bytes, server
int, params Map[str, str] }`: the fields of the `net.Request`
(`std.net.9`) and the route's parameters by name. `req.param(name) str?` is
one parameter or `none`. `req.json() Json ! JsonError` reads the body as
JSON. `req.form() Map[str, str]` reads it as
`application/x-www-form-urlencoded`: pieces split at `&`, then at the first
`=`, with `+` as a space and `%XX` decoded; a piece without `=` has the
value `""` and a repeated name keeps its last value. `req.stop()` stops the
server, as on a `net.Request` (`http.serve.1`). *(ADR 1217)*

**http.route.3** — When no route's pattern matches the path, the first
mount from `r.static(prefix, dir)` whose prefix segments begin the path
answers with the file under `dir` named by the rest, as `serve_dir` does
(`http.serve.2`); with no such mount the answer is `not_found()`. When a
pattern matches but no route of it takes the method, the answer is `405`
with `allow`, the methods of the matching routes in the order added
(`HEAD` after them when `GET` is one); mounts are not tried then. *(ADR
1217)*

**http.route.4** — `r.handle(request net.Request) net.Response` is the
answer the router gives, without a server; a file's body is its text, with
bytes that aren't UTF-8 as U+FFFD. `r.run(srv) ! Trapped` answers every
request on `srv` until `srv.close()` or `req.stop()`, with the workers of
`http.run` (a file is sent as its bytes), and `r.serve(addr) ! NetError |
Trapped` listens first. `r.on_request(hook fn(http.Req, net.Response))`
calls `hook` once for each answer, after the handler and before it is sent;
for a file the response it sees has an empty body, and for a `404` or
`405` the `Req` has no `params`. A later call replaces the hook. Routes
added after `run` starts are not seen by it (the router is a value).
*(ADR 1217)*

**http.route.5** — The response helpers: `text(status, body)` with
`content-type: text/plain; charset=utf-8`, `html(status, body)` with
`text/html; charset=utf-8`, `json(status, value Json)` with
`application/json` and the compact text (`json.print.1`), `redirect(to)` a
`302` with `location: to` and no body, and `not_found()` is `text(404, "not
found")`. `mime_type(ext) str` is the content type `serve_dir` gives a file
extension (`http.serve.2`), with or without its dot, in any case.
*(ADR 1217)*

**http.gzip.1** — `r.gzip()` (needs `mut r`) makes `r.run` and `r.serve` answer
with gzip (RFC 1952, `compress.gzip`) when the request's `accept-encoding`
allows it. `http.accepts_gzip(header str) bool` is that test: `gzip` or
`x-gzip` with a `q` that is not zero, in any case, or `*` when gzip is not
named; a header with no such coding, and an empty one, say no. *(ADR 1230)*

**http.gzip.2** — Only a body of text is compressed: a `content-type` that starts
with `text/` or contains `json`, `xml`, `javascript` or `wasm`, a body of 256
bytes or more (a file's bytes, or the response's `body`), no `content-encoding`
already, and a status other than `1xx`, `204`, `206` and `304`. Such a response
always gets `vary: accept-encoding` (added to an existing `vary`), compressed
or not; a compressed one has `content-encoding: gzip` and the gzip data as its
body, with `content-length` of that. Other responses are sent as they are.
*(ADR 1230)*

**http.gzip.3** — The `on_request` hook sees the response before it is
compressed, and `r.handle` never compresses (its body is a `str`). `http.run`,
`http.serve`, `serve_dir` and `run_dir` do not compress. *(ADR 1230)*

```ail rules=http.route.1,http.route.2,http.route.3,http.route.4,http.route.5
use std.net
use std.net.http

fn ask(method str, p str, body str) net.Request
    net.Request(method: method, path: p, query: {}, headers: {}, body: body, peer: "", data: b"", server: -1)

fn user(r http.Req) net.Response
    id := r.param("id") else "?"
    http.text(200, "user {id}")

fn make(r http.Req) net.Response
    form := r.form()
    http.text(201, "made {form["name"]}")

fn serve_routes(r http.Router, srv net.Server) ! Trapped
    r.run(srv)?

fn main() !
    mut r := http.router()
    r.get("/users/:id", user)
    r.post("/users", make)
    r.get("/files/*rest", |q| http.text(200, q.param("rest") else ""))
    print(r.handle(ask("GET", "/users/a%20b", "")).body)
    print(r.handle(ask("GET", "/files/x/y.txt", "")).body)
    print(r.handle(ask("GET", "/nope", "")).status)
    no := r.handle(ask("DELETE", "/users/7", ""))
    print("{no.status} {no.headers["allow"]}")
    print(http.mime_type("PNG"))
    srv := net.serve("127.0.0.1:0")?
    t := spawn serve_routes(r, srv)
    print(net.post("http://{srv.addr()}/users", "name=a+b", "application/x-www-form-urlencoded")?.body)
    srv.close()
    t.wait()?
```

```ail rules=http.gzip.1,http.gzip.2,http.gzip.3
use std.net
use std.net.http
use std.compress

fn page(r http.Req) net.Response
    http.text(200, "abc ".repeat(200))

fn serve_routes(r http.Router, srv net.Server) ! Trapped
    r.run(srv)?

fn main() !
    print(http.accepts_gzip("gzip, deflate"))
    print(http.accepts_gzip("gzip;q=0"))
    mut r := http.router()
    r.get("/", page)
    r.gzip()
    srv := net.serve("127.0.0.1:0")?
    t := spawn serve_routes(r, srv)
    mut o := net.options()
    o.headers["accept-encoding"] = "gzip"
    a := net.request_bytes("GET", "http://{srv.addr()}/", o)?
    print(a.headers["content-encoding"])
    print(compress.gunzip(a.data)?.text().len())
    srv.close()
    t.wait()?
```

## Client helpers

**http.client.1** — `get_json(link str) Json ! net.NetError | http.HttpError
| JsonError` sends `GET` with `accept: application/json` and reads a 2xx
body as JSON; `post_json(link str, value Json)` sends the text of `value` as
`application/json` and reads the answer the same way. A status outside 200
to 299 fails with `HttpError { link str, status int, body str }`, a body that
is not JSON with `JsonError`. They are `net.request` with those options
(`std.net.6`). *(ADR 1233)*

**http.client.2** — `post_form(link str, pairs List[(str, str)]) net.Response
! net.NetError` sends `POST` with `content-type:
application/x-www-form-urlencoded` and the body `url.build_query(pairs)`
(`std.url.9`): each name and value encoded by `url.encode_form`, in order. Any
status is an answer. *(ADR 1233)*

**http.client.3** — `multipart(parts List[Part], boundary str = "")
(bytes, str)` is the body and the `Content-Type` of a `multipart/form-data`
request (RFC 7578). `struct Part { name str, filename str, content_type str,
data bytes }`; `field(name, value)` is a text field and `file_part(name,
filename, content_type, data)` a file. Each part is `--BOUNDARY`, a
`Content-Disposition: form-data; name="..."` header (with `; filename="..."`
when `filename` is not ""), a `Content-Type` header when `content_type` is
not "", an empty line, the data and CRLF; the body ends with
`--BOUNDARY--`. In a name or file name `"`, CR and LF are written `%22`,
`%0D` and `%0A`. With no `boundary` a random one is used that occurs in no
part's data. `post_multipart(link, parts) net.Response ! net.NetError` sends
that body with `POST`. *(ADR 1233)*

**http.client.4** — `retry(attempts int, base_ms int, call fn() net.Response
! net.NetError) net.Response ! net.NetError` calls `call` up to `attempts`
times (at least 1). An answer with a status below 500 is returned; so is a
failure that is not of kind `timed_out`, `refused` or `closed`. After
a 5xx status or such a failure it waits and calls again; the last call's
answer is returned as it is, a 5xx response or the failure included. The wait
before retry `n` is `backoff_ms(base_ms, n)` jittered to between half and
all of it. `call` should be safe to repeat. *(ADR 1233)*

**http.client.5** — `backoff_ms(base_ms int, attempt int) int` is `base_ms`
doubled for each retry before retry `attempt` (1 is the first), at most
30000; a negative `base_ms` counts as 0. *(ADR 1233)*

```ail rules=http.client.1,http.client.2,http.client.3,http.client.4,http.client.5
use std.net
use std.net.http

fn nums(r http.Req) net.Response
    http.text(200, "[1, 2]")

fn serve_routes(r http.Router, srv net.Server) ! Trapped
    r.run(srv)?

fn main() !
    mut r := http.router()
    r.get("/nums", nums)
    r.post("/form", |q| http.text(200, q.form()["a"]))
    srv := net.serve("127.0.0.1:0")?
    t := spawn serve_routes(r, srv)
    base := "http://{srv.addr()}"
    print(http.get_json("{base}/nums")?.to_text())
    print(http.post_form("{base}/form", [("a", "x y")])?.body)
    body, ctype := http.multipart([http.field("n", "v")], boundary: "B")
    print("{ctype} {body.len()}")
    got := http.retry(3, 10, || net.get("{base}/nums"))?
    print(got.status)
    print(http.backoff_ms(100, 3))
    srv.close()
    t.wait()?
```
