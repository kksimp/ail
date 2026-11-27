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

## Middleware

**http.mw.1** — `r.use(mw)` adds a middleware, `fn(http.Req, http.Next)
net.Response`, to a router (`mut r`). Details: [below](#more-on-httpmw1). *(ADR 1276)*

**http.mw.2** — `r.group(prefix, sub)` adds the routes and static mounts of
the router `sub` under `prefix`, at the position of the call, so a group is
built as a router of its own with its own `use`. Details: [below](#more-on-httpmw2). *(ADR 1276)*

**http.mw.3** — `http.recover` is a middleware that runs the rest of the
chain as a task and, when it traps, answers `500` with the body `internal
server error` and logs the method, path and trap message with `log.err`.
Put it first to cover the other middleware. *(ADR 1276)*

**http.mw.4** — `http.request_id` gives the `Req` an `x-request-id` header
and sets the same header on the response: the client's, when it sent one of at
most 64 characters, else 16 random hex digits. *(ADR 1276)*

**http.mw.5** — `http.log_requests` logs `METHOD PATH STATUS MSms` with
`log.info` after the chain has answered. *(ADR 1276)*

```ail rules=http.mw.1,http.mw.2,http.mw.3,http.mw.4,http.mw.5
use std.net
use std.net.http

fn ask(p str, headers Map[str, str]) net.Request
    net.Request(method: "GET", path: p, query: {}, headers: headers, body: "", peer: "", data: b"", server: -1)

fn tag(req http.Req, next http.Next) net.Response
    mut resp := next.run(req)
    resp.headers["x-tag"] = "seen"
    resp

fn guard(req http.Req, next http.Next) net.Response
    if not req.headers.contains("authorization")
        return http.text(401, "denied")
    next.run(req)

fn main() !
    mut admin := http.router()
    admin.use(guard)
    admin.get("/stats", |q| http.text(200, "stats"))
    mut r := http.router()
    r.use(http.recover)
    r.use(http.request_id)
    r.use(http.log_requests)
    r.use(tag)
    r.get("/hi", |q| http.text(200, "hi"))
    r.group("/admin", admin)
    print(r.handle(ask("/hi", {})).headers["x-tag"])
    print(r.handle(ask("/admin/stats", {})).status)
    print(r.handle(ask("/admin/stats", {"authorization": "x"})).body)
    print(r.handle(ask("/hi", {})).headers["x-request-id"].len())
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
request (RFC 7578). Details: [below](#more-on-httpclient3). *(ADR 1233)*

**http.client.4** — `retry(attempts int, base_ms int, call fn() net.Response
! net.NetError) net.Response ! net.NetError` calls `call` up to `attempts`
times (at least 1). An answer with a status below 500 is returned; so is a
failure that is not of kind `timed_out`, `refused` or `closed`. Details: [below](#more-on-httpclient4). *(ADR 1233)*

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

### `http.client` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on http.client.3

`struct Part { name str, filename str, content_type str, data bytes }`; `field(name, value)` is a text field and `file_part(name, filename, content_type, data)` a file. Each part is `--BOUNDARY`, a `Content-Disposition: form-data; name="..."` header (with `; filename="..."` when `filename` is not ""), a `Content-Type` header when `content_type` is not "", an empty line, the data and CRLF; the body ends with `--BOUNDARY--`. In a name or file name `"`, CR and LF are written `%22`, `%0D` and `%0A`. With no `boundary` a random one is used that occurs in no part's data. `post_multipart(link, parts) net.Response ! net.NetError` sends that body with `POST`.

#### More on http.client.4

After a 5xx status or such a failure it waits and calls again; the last call's answer is returned as it is, a 5xx response or the failure included. The wait before retry `n` is `backoff_ms(base_ms, n)` jittered to between half and all of it. `call` should be safe to repeat.
**http.route.6** — `r.upgrade(pattern, handler)` adds a `GET` route (it needs `mut r`) whose handler is `fn(http.Req, net.Conn)`: it gets the request and the connection (`ex.hijack()`, `std.net.53`), writes whatever it likes (a WebSocket handshake with `websocket.upgrade`, or `HTTP/1.1 200 OK` and a stream of events) and owns the connection until it returns, when the router closes it. Details: [below](#more-on-httproute6). *(ADR 1280)*
**http.route.7** — A `GET` is handed to an upgrade route when the first route whose pattern and method match it is one; an earlier plain route wins. Details: [below](#more-on-httproute7). *(ADR 1280)*
**http.route.8** — `r.on_request` is not called for an upgrade route, since the router sees no response. *(ADR 1280)*
```ail rules=http.route.6,http.route.7,http.route.8,std.net.51,std.net.53
use std.net
use std.net.http
use std.net.websocket

fn chat(req http.Req, conn net.Conn)
    mut ws := websocket.upgrade(conn, req.method, req.path, req.headers) else err
        return
    while true
        m := ws.recv() else err
            return
        if m.kind == websocket.Kind.Text
            ws.send_text("echo {m.text()}") else err
                return
        elif m.kind == websocket.Kind.Close
            return

fn serve_routes(r http.Router, srv net.Server) ! Trapped
    r.run(srv)?

fn main() !
    mut r := http.router()
    r.get("/hello", |q| http.text(200, "hello"))
    r.upgrade("/chat", chat)
    print(r.handle(net.Request(method: "GET", path: "/chat", query: {}, headers: {}, body: "", peer: "", data: b"", server: -1)).status)
    srv := net.serve("127.0.0.1:0")?
    t := spawn serve_routes(r, srv)
    print(net.get("http://{srv.addr()}/hello")?.body)
    mut ws := websocket.connect("ws://{srv.addr()}/chat")?
    ws.send_text("hi")?
    print(ws.recv()?.text())
    ws.close(1000, "bye")
    srv.close()
    t.wait()?
```

## Static files
`use std.net.fileserve` is the module `serve_dir` and `r.static` answer with;
`http.run_dir_with`, `http.serve_dir_with` and `r.static_with` take its
`Options`. A program may call `fileserve.answer` itself.
**http.file.1** — `fileserve.answer(method, rel, root, headers, opts) Answer`
is the answer to `method` for the file `rel` (`/`-separated segments, as sent)
under `root`, given the request's lowercase `headers`. Details: [below](#more-on-httpfile1). *(ADR 1287)*
**http.file.2** — `fileserve.mime_type(ext)` is the content type for an
extension, with or without its dot, in any case: the table of `http.serve.2`
and `map`, `mjs`, `webmanifest`, `csv`, `yaml`, `yml`, `avif`, `bmp`, `tif`,
`tiff`, `zip`, `gz`, `tar`, `woff`, `woff2`, `ttf`, `otf`, `mp3`, `ogg`,
`wav`, `flac`, `m4a`, `mp4`, `webm` and `mov`; `application/octet-stream`
otherwise. Details: [below](#more-on-httpfile2). *(ADR 1287)*
**http.file.3** — A `200` or `206` answer has `etag` (a weak tag from the size
and modification time of the file sent), `last-modified` (`Wed, 21 Oct 2015
07:28:00 GMT`, whole seconds) and `accept-ranges: bytes`. A request with
`if-none-match` is answered `304` with those headers and no body when it
lists the tag (weak comparison) or `*`; without `if-none-match`, a valid
`if-modified-since` at or after the modification time gives `304`; an
unreadable date is ignored. *(ADR 1287)*
**http.file.4** — With `Options.precompressed` (the default), a file `NAME`
with a neighbour `NAME.br` or `NAME.gz` is answered with the neighbour's bytes
and `content-encoding` `br` or `gzip` when `accept-encoding` names that
coding with a `q` above zero (`br` first), the `content-type` still that of
`NAME`, and the validators those of the neighbour. Details: [below](#more-on-httpfile4). *(ADR 1287)*
**http.file.5** — `range: bytes=A-B`, `A-` or `-N` on a `GET` gives `206` with
that part of the body (the sent representation, so of a neighbour when
http.file.4 chose one) and `content-range: bytes A-B/SIZE`; `B` past the end
is the end. Details: [below](#more-on-httpfile5). *(ADR 1287)*
**http.file.6** — A path that names a directory is answered with the file
`Options.index` in it, `404` when there is none. The answer to `HEAD` has the
headers of `GET`. *(ADR 1287)*
**http.file.7** — `fileserve.options()` is `Options(cache_control: "", index:
"index.html", precompressed: true)`, all `pub` fields. A `cache_control` that
isn't empty is sent as `cache-control` on `200`, `206` and `304`.
`http.serve_dir`, `run_dir` and `r.static` use `options()`. *(ADR 1287)*
```ail rules=http.file.1,http.file.2,http.file.3,http.file.4,http.file.5,http.file.6,http.file.7
use std.fs
use std.net.fileserve

fn main() !
    fs.make_dir("http_file_spec.tmp")?
    fs.write_text("http_file_spec.tmp/a.txt", "0123456789")?
    fs.write_text("http_file_spec.tmp/index.html", "home")?
    mut opts := fileserve.options()
    opts.cache_control = "max-age=60"
    root := "http_file_spec.tmp"
    a := fileserve.answer("GET", "/a.txt", root, Map[str, str](), opts)
    print("{a.status} {a.headers["content-type"]} {a.headers["cache-control"]}")
    again := fileserve.answer("GET", "/a.txt", root, {"if-none-match": a.headers["etag"]}, opts)
    print(again.status)
    part := fileserve.answer("GET", "/a.txt", root, {"range": "bytes=2-4"}, opts)
    print("{part.status} {part.data.text()} {part.headers["content-range"]}")
    print(fileserve.answer("GET", "/a.txt", root, {"range": "bytes=99-"}, opts).status)
    print(fileserve.answer("GET", "/", root, Map[str, str](), opts).data.text())
    print(fileserve.answer("GET", "/../x", root, Map[str, str](), opts).status)
    print(fileserve.mime_type("woff2"))
    fs.remove("{root}/a.txt")?
    fs.remove("{root}/index.html")?
    fs.remove(root)?
```

### `http.file` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on http.file.1

`Answer` has `status int`, `headers Map[str, str]` and `data bytes`. Only `GET` and `HEAD` are served; any other method is `405` with `allow: GET, HEAD`. A segment that is `..`, or holds a backslash or a colon, is `403`; a missing file, or a directory with no index file, is `404`. A symbolic link inside `root` is followed.

#### More on http.file.2

`http.mime_type` is the same function.

#### More on http.file.4

A file with a neighbour also gets `vary: accept-encoding`.

#### More on http.file.5

A start at or past the size, or `-0`, is `416` with `content-range: bytes */SIZE`. A range of another unit, one that is malformed, one with `B` before `A`, and a list of ranges are ignored: the answer is `200` with the whole body. `if-range` makes the range count only when it equals `last-modified`; an entity tag never does, as the tag is weak.

### `http.mw` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on http.mw.1

The middleware of a router run around the handler of every route that matches, and around the `404` and `405` answers (not around a file of `r.static`); the first one added is the outermost, and the order of `use` and `get` calls doesn't matter. A middleware does its work before the handler, then calls `next.run(req)`, which runs the middleware after it and then the handler, and may change the `Req` it passes on and the response it gets back; it may also answer without calling `next`. `r.on_request`'s hook sees the answer after the chain.

#### More on http.mw.2

The middleware of `sub` run around its routes, inside those of `r`. `sub` is copied: later changes to it are not seen, and its hook and gzip setting are ignored. Groups nest.

### `http.route` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on http.route.6

The pattern is as in `http.route.1`.

#### More on http.route.7

The handler runs in a task of its own, so a long exchange holds none of the router's workers, and `r.run(srv)` returns once `srv` is closed and every such handler has returned. `r.handle(request)` and a `HEAD` request answer an upgrade route `426` with the body `upgrade required`.
