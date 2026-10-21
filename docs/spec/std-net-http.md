# Standard library: `std.net.http` server

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in the bootstrap (ADRs
> 0754 and 0755). The server itself is `std.net.9`–`11`, `22` and `24`, and
> the handler form is `std.net.18`; this chapter adds what an agent needs to
> build a small service on top of them.

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
`ico`, `pdf`, `wasm`) and is `application/octet-stream` otherwise. A path with
a `..` segment gets `403`, a missing file or an unreadable one `404`, and any
other method `405`. The path is used as sent: `%XX` is not decoded, and a
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
