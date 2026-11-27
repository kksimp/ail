# Standard library: `std.net.sse`, Server-Sent Events

Chapter key: `std`

> **Status:** Drafted. Native only (ADR TBD). Written in AIL over `net.Conn`
> (`std.net.12`, `std.net.16`); the format is the `text/event-stream` format
> of the WHATWG HTML standard, section 9.2. `use std.net.sse` is always
> written, and the module is used as `sse` (`std.use.4`).

**std.sse.1** — An event is `struct Event { name str, data str, id str, retry int }`.
`sse.event(data)` is an event without a name (its `name` is `message`),
`sse.named(name, data)` one with a name, `ev.with_id(id)` and `ev.with_retry(ms)`
return a copy with an id or a reconnection time. `id` is `""` and `retry` is
-1 when there is none.

```ail
use std.net.sse

fn main()
    ev := sse.named("tick", "1").with_id("7")
    print("{ev.name} {ev.data} {ev.id} {ev.retry}")
```

**std.sse.2** — `ev.encode() str` is the wire form: `event: NAME` unless the
name is empty or `message`, `id: ID` unless empty, `retry: MS` unless
negative, one `data: LINE` for each line of the data (CRLF, CR and LF each
end a line; empty data gives one empty `data:` line), then a blank line.
Line breaks inside the name and the id are dropped.

```ail
use std.net.sse

fn main()
    print(sse.event("a\nb").encode().replace("\n", "/"))
```

**std.sse.3** — `sse.comment(text) str` is `: TEXT` and a blank line, with line
breaks in the text dropped. A client ignores a comment.

```ail
use std.net.sse

fn main()
    print(sse.comment("keepalive").replace("\n", "/"))
```

**std.sse.4** — `sse.listen(addr str) Server ! SseError` listens as
`net.listen` does (`std.net.12`); `s.addr()` and `s.close()` match `Listener`.
`s.accept() Request ! SseError` waits for a client and reads its request,
which is returned unanswered as `struct Request { method, path, headers,
last_event_id }`: `path` has the query, `headers` has names in lower case,
`last_event_id` is the `Last-Event-ID` header or `""`. Bytes that are not an
HTTP/1.x request line are answered `400` and fail with kind `Protocol`; the
server still listens, so call `accept` again. A server is plain `http`: there
is no TLS server (`std.net.13`). `sse` does not route: a program looks at
`path` itself, or runs one `Server` for each address.

```ail
use std.net.sse

fn main()
    srv := sse.listen("127.0.0.1:0") else err
        print("failed: {err.message}")
        return
    print(srv.addr().starts_with("127.0.0.1:"))
    srv.close()
```

**std.sse.5** — `r.open() Stream ! SseError` answers `200` with `Content-Type:
text/event-stream`, `Cache-Control: no-cache`, `Connection: close` and
`X-Accel-Buffering: no`, no `Content-Length`, and a first comment `: open`; the
body ends when the stream closes. `r.refuse(status int, body str)` answers
with that status (`400`, `403`, `404`, `405`, `406`, `429` and `503` have their
reason phrase, any other `Error`), the text `body` as `text/plain` and closes the
connection. A `Stream` has the fields `path`, `headers` and `last_event_id` of its request.

```ail
use std.net.sse

fn refuse_one(srv sse.Server)
    r := srv.accept() else err
        return
    r.refuse(404, "no such stream")

fn main()
    srv := sse.listen("127.0.0.1:0") else err
        return
    t := spawn refuse_one(srv)
    _ := sse.connect("http://{srv.addr()}/") else err
        print("{err.kind}: {err.message}")
    t.wait() else err
        return
    srv.close()
```

**std.sse.6** — `st.send(ev Event) ! SseError` writes `ev.encode()`;
`st.note(text str) ! SseError` writes `sse.comment(text)`; `st.peer() str` and
`st.close()` match `Conn`. A write to a client that has gone fails with kind
`Closed` or `Net`. A `Stream` is used from one task at a time.

```ail
use std.net.sse

fn serve(srv sse.Server)
    r := srv.accept() else err
        return
    st := r.open() else err
        return
    st.send(sse.event("hello")) else err
        return
    st.close()

fn main()
    srv := sse.listen("127.0.0.1:0") else err
        return
    t := spawn serve(srv)
    mut c := sse.connect("http://{srv.addr()}/") else err
        return
    ev := c.next() else err
        return
    print(ev.data)
    c.close()
    t.wait() else err
        return
    srv.close()
```

**std.sse.7** — `st.pump(events Channel[Event], keepalive_ms int) ! SseError`
sends every event received from `events` and, each time `keepalive_ms` (at
least 1) pass with no event, the comment `keepalive` (so a proxy sees traffic and
a client that left is noticed). When the channel is closed and empty, `pump`
closes the stream and returns. When a write fails it closes the stream and
fails with the write's kind. Several tasks may send on the channel: it is the
way to feed one stream from many producers.

```ail
use std.net.sse

fn feed(srv sse.Server)
    r := srv.accept() else err
        return
    st := r.open() else err
        return
    ch := Channel[sse.Event](2)
    ch.send(sse.event("a"))
    ch.send(sse.event("b"))
    ch.close()
    st.pump(ch, 50) else err
        return

fn main()
    srv := sse.listen("127.0.0.1:0") else err
        return
    t := spawn feed(srv)
    mut c := sse.connect("http://{srv.addr()}/") else err
        return
    c.set_retries(0)
    for _ in 0..2
        ev := c.next() else err
            return
        print(ev.data)
    t.wait() else err
        return
    c.close()
    srv.close()
```

**std.sse.8** — `sse.connect(url str) Client ! SseError` connects to
`http://HOST[:PORT][/PATH][?QUERY]` or `https://...` (TLS 1.3, `std.net.13`) and
sends `GET` with `Accept: text/event-stream`, `Cache-Control: no-cache` and
`Accept-Encoding: identity`. It succeeds only when the answer is `200` with a
`Content-Type` that starts with `text/event-stream`; any other status or type
fails with kind `Status`, a URL of another form or a port outside 1 to 65535 with
`BadUrl`. The body is read as it comes, plain (until the connection closes) or
`Transfer-Encoding: chunked`. `c.headers` is the response's headers, names in lower
case. `c.set_retries(n int)` is how many times in a row `next` reconnects (default
5; 0 turns it off), `c.set_retry(ms int)` the wait before a reconnect until the
server sends `retry:` (default 3000), `c.set_timeout(ms int)` how long a read may
wait (default 30000, as `Conn.set_timeout`; a wait that long fails with kind
`TimedOut`), and `c.close()`.

```ail
use std.net.sse

fn main()
    _ := sse.connect("ftp://example.test/") else err
        print(err.kind)
        return
```

**std.sse.9** — `c.next() Event ! SseError` waits for the next event. Lines end
with LF or CRLF; a leading byte-order mark is dropped; a line that starts with `:`
is a comment; a line is `FIELD: VALUE` (one space after the colon is dropped) or a
field with no value. `data` lines are joined with `\n`; `event` sets the name
(`message` when empty); `id` sets `c.last_event_id()`; `retry` with only digits
sets the reconnect wait at once and is also `ev.retry` of the event it comes in;
other fields are ignored. A blank line ends an event, which is returned when it
has at least one `data` line (else it is dropped); `ev.id` is the last id seen so
far. An event that is not finished when the connection ends is dropped.

```ail
use std.net.sse

fn serve(srv sse.Server)
    r := srv.accept() else err
        return
    st := r.open() else err
        return
    st.send(sse.named("tick", "a\nb").with_id("5")) else err
        return
    st.close()

fn main()
    srv := sse.listen("127.0.0.1:0") else err
        return
    t := spawn serve(srv)
    mut c := sse.connect("http://{srv.addr()}/") else err
        return
    ev := c.next() else err
        return
    print("{ev.name} {ev.data.replace("\n", "/")} {c.last_event_id()}")
    c.set_retries(0)
    c.close()
    t.wait() else err
        return
    srv.close()
```

**std.sse.10** — When the stream ends, or a read or write fails with kind `Closed`,
`TimedOut` or `Net`, `next` waits `retry` milliseconds, connects again to the same
URL with `Last-Event-ID: ID` when an id was seen, and goes on; the count of
failures in a row (it starts again at each event) is limited by `set_retries`, and
the call then fails with the last failure's kind. A failure of kind `Status`,
`Protocol` or `BadUrl` is never retried: a server that answers `204` or any
other non-`200` status on a reconnect ends the loop, as the standard says. A
failure is `error struct SseError { kind SseKind, message str }`; `SseKind` is
`BadUrl`, `Protocol`, `Status`, `Closed`, `TimedOut` or `Net`. A `Client` is used
from one task at a time. Not supported: redirects, gzip, `EventSource`'s
`withCredentials`, and CORS headers (add them with a proxy). Every function has the
effects `net` and `io`; `listen` and `Server.accept` also `net.serve` (`std.net.14`).

```ail
use std.net.sse

fn main()
    print(sse.SseKind.Status)
```
