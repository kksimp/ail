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
    t := spawn r.run(srv)
    print(net.post("http://{srv.addr()}/users", "name=a+b", "application/x-www-form-urlencoded")?.body)
    srv.close()
    t.wait()?
```
