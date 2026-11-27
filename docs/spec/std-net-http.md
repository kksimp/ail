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

## Static files

`use std.net.fileserve` is the module `serve_dir` and `r.static` answer with;
`http.run_dir_with`, `http.serve_dir_with` and `r.static_with` take its
`Options`. A program may call `fileserve.answer` itself.

**http.file.1** — `fileserve.answer(method, rel, root, headers, opts) Answer`
is the answer to `method` for the file `rel` (`/`-separated segments, as sent)
under `root`, given the request's lowercase `headers`. `Answer` has `status
int`, `headers Map[str, str]` and `data bytes`. Only `GET` and `HEAD` are
served; any other method is `405` with `allow: GET, HEAD`. A segment that is
`..`, or holds a backslash or a colon, is `403`; a missing file, or a
directory with no index file, is `404`. A symbolic link inside `root` is
followed. *(ADR TBD)*

**http.file.2** — `fileserve.mime_type(ext)` is the content type for an
extension, with or without its dot, in any case: the table of `http.serve.2`
and `map`, `mjs`, `webmanifest`, `csv`, `yaml`, `yml`, `avif`, `bmp`, `tif`,
`tiff`, `zip`, `gz`, `tar`, `woff`, `woff2`, `ttf`, `otf`, `mp3`, `ogg`,
`wav`, `flac`, `m4a`, `mp4`, `webm` and `mov`; `application/octet-stream`
otherwise. `http.mime_type` is the same function. *(ADR TBD)*

**http.file.3** — A `200` or `206` answer has `etag` (a weak tag from the size
and modification time of the file sent), `last-modified` (`Wed, 21 Oct 2015
07:28:00 GMT`, whole seconds) and `accept-ranges: bytes`. A request with
`if-none-match` is answered `304` with those headers and no body when it
lists the tag (weak comparison) or `*`; without `if-none-match`, a valid
`if-modified-since` at or after the modification time gives `304`; an
unreadable date is ignored. *(ADR TBD)*

**http.file.4** — With `Options.precompressed` (the default), a file `NAME`
with a neighbour `NAME.br` or `NAME.gz` is answered with the neighbour's bytes
and `content-encoding` `br` or `gzip` when `accept-encoding` names that
coding with a `q` above zero (`br` first), the `content-type` still that of
`NAME`, and the validators those of the neighbour. A file with a neighbour
also gets `vary: accept-encoding`. *(ADR TBD)*

**http.file.5** — `range: bytes=A-B`, `A-` or `-N` on a `GET` gives `206` with
that part of the body (the sent representation, so of a neighbour when
http.file.4 chose one) and `content-range: bytes A-B/SIZE`; `B` past the end
is the end. A start at or past the size, or `-0`, is `416` with `content-range:
bytes */SIZE`. A range of another unit, one that is malformed, one with `B`
before `A`, and a list of ranges are ignored: the answer is `200` with the
whole body. `if-range` makes the range count only when it equals
`last-modified`; an entity tag never does, as the tag is weak. *(ADR TBD)*

**http.file.6** — A path that names a directory is answered with the file
`Options.index` in it, `404` when there is none. The answer to `HEAD` has the
headers of `GET`. *(ADR TBD)*

**http.file.7** — `fileserve.options()` is `Options(cache_control: "", index:
"index.html", precompressed: true)`, all `pub` fields. A `cache_control` that
isn't empty is sent as `cache-control` on `200`, `206` and `304`.
`http.serve_dir`, `run_dir` and `r.static` use `options()`. *(ADR TBD)*

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
