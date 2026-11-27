# Standard library: `std.net.sse`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD); native only.

Server-Sent Events, the `text/event-stream` format of the WHATWG HTML
standard (section 9.2): a parser and encoder, a client that reconnects, and a
server that streams events. The module is written in AIL over `net.Conn`
(`std.net.12`, `std.net.16`). `use std.net.sse` is always written (`std.use.4`)
and names it `sse`.

## Events

**std.net.sse.1** — `Event { name str, data str, id str, retry int }`: `name`
is the `event:` field ("" for a plain message), `data` the message, `id` the
event id ("" for none), `retry` a reconnection time in milliseconds (-1 for
none). `sse.event(data)` makes a message and `sse.named(name, data)` a named
event; `ev.with_id(id)` and `ev.with_retry(ms)` return a copy with that field.
*(ADR TBD)*

**std.net.sse.2** — `sse.encode(ev) str` is the text of `ev` on a stream: an
`event:` line when `name` is not empty, one `data:` line for each line of `data`
(line ends `\n`, `\r\n` and `\r` all split), an `id:` line when `id` is not
empty, a `retry:` line when `retry` is not negative, and a blank line. A newline
in `name` or `id` becomes a space. `sse.response(events List[Event])
net.Response` is a response of status 200 with `content-type:
text/event-stream` and `cache-control: no-cache` whose body is the events
encoded, which a `std.net.http` handler or router returns for a finite
stream. *(ADR TBD)*

```ail rules=std.net.sse.1,std.net.sse.2
use std.net.sse

fn main()
    ev := sse.named("tick", "a\nb").with_id("5")
    print(sse.encode(ev).replace("\n", "|"))   # event: tick|data: a|data: b|id: 5||
    r := sse.response([sse.event("x")])
    print("{r.status} {r.body.replace("\n", "|")}")   # 200 data: x||
```

**std.net.sse.3** — The parser follows the standard. `sse.parser()` returns a
`Parser`; `p.feed(chunk bytes) List[Event]` takes the bytes of a stream in
pieces of any size (a character or a line may be split between pieces) and
returns the events they complete. Lines end in `\n`, `\r\n` or `\r`; a
leading byte order mark is skipped; a line that starts with `:` is a comment; a
field's value loses one leading space; a line without `:` is a field with the
value ""; `data` lines are joined by `\n`; an event without a `data` line is not
returned; fields other than `event`, `data`, `id` and `retry` are ignored; an
`id` with a NUL is ignored; `retry` counts only when all digits. An event
carries `p.last_id`, the last `id:` seen, even when it set none itself.
`p.retry` is the last `retry:` value not yet taken, -1 when none (a `retry:`
line takes effect without an event). `sse.parse(text str) List[Event]` is
the events of a whole stream; an event not ended by a blank line is dropped.
*(ADR TBD)*

```ail rules=std.net.sse.3
use std.net.sse

fn main()
    for ev in sse.parse("event: ping\ndata: x\nid: 7\n\n: note\ndata\n\n")
        print("[{ev.name}] {ev.data} {ev.id}")   # [ping] x 7, then [] "" 7
```

## Client

**std.net.sse.4** — `sse.connect(address str) Client ! SseError` opens the
stream at an `http://` or `https://` address (TLS 1.3, `std.net.13`; default
ports 80 and 443) with a `GET` that sends `Accept: text/event-stream`, and
returns once the server has answered `200` with a `text/event-stream` content
type. `sse.connect_with(address, headers Map[str, str])` also sends `headers` on
every request (an `Authorization` header, for one). Chunked bodies are decoded.
Any other answer fails with kind `Handshake` (and the message names the status
line); an address that is not `http://` or `https://`, or has a bad port, fails
with `BadUrl`; a failure to connect or to read fails with `Net`, `TimedOut` or
`Closed`. Redirects are not followed. *(ADR TBD)*

**std.net.sse.5** — `c.next() Event ! SseError` waits for the next event. When
the connection ends or fails, it waits `c.retry` milliseconds (3000 until the
server sends `retry:`; `c.set_retry(ms)` changes it), opens the stream again
with `Last-Event-ID: c.last_id` (the last event id received), and goes on; up
to `c.set_max_retries(n)` reopens in a row (default 5, 0 never reconnects), and
an event received starts the count again. When the retries are used up it fails
with the kind of the last failure. A reopen the server answers with anything but
`200` and an event stream fails the call at once (kind `Handshake`), as `204`
tells a client to stop. `c.connects` is how many times a connection was opened.
`c.set_timeout(ms)` is how long a read may make no progress (default 30000; kind
`TimedOut`, which is also reconnected). `c.close()` stops the client: `next`
fails with kind `Closed`. *(ADR TBD)*

```ail norun rules=std.net.sse.4,std.net.sse.5
use std.net.sse

fn main() ! sse.SseError
    mut c := sse.connect("http://127.0.0.1:8080/events")?
    c.set_max_retries(2)
    for _ in 0..3
        ev := c.next()?
        print("{ev.name} {ev.data} {c.last_id}")
    c.close()
```

## Server

**std.net.sse.6** — `sse.listen(addr str) Server ! SseError` listens as `net.listen`
does (`std.net.12`); `s.addr()` and `s.close()` match `Listener`. `s.accept()
Request ! SseError` waits for a client and reads its request head (at most 64
KiB). A request that is not `HTTP/1.x` is answered `400`, one that is not `GET`
`405`, and the call fails with kind `BadRequest`; call `accept` again. A
`Request` has `method`, `path` (before `?`), `query` (decoded, first value of a
repeated key), `headers` (names in lower case), `last_event_id` (the
`Last-Event-ID` header, "" for a first connection) and `peer`. It is not
answered until one of `std.net.sse.7`. *(ADR TBD)*

**std.net.sse.7** — `req.stream() Stream ! SseError` answers `200` with `Content-Type:
text/event-stream`, `Cache-Control: no-cache`, `X-Accel-Buffering: no` and
`Connection: close` (the body ends when the connection does) and returns the
`Stream`; `req.stream_with(extra Map[str, str])` adds headers, for CORS.
`req.reject(status int, why str)` answers with that status and `why` as plain
text and closes; `req.close()` closes without an answer. A `Stream` has
`last_event_id`, `path`, `query` and `peer` of its request. *(ADR TBD)*

**std.net.sse.8** — On a `Stream`: `s.send(ev)` writes `sse.encode(ev)`;
`s.send_data(data)` a message; `s.comment(text)` a comment line (`: text`)
that clients ignore, used as a heartbeat; `s.retry(ms)` tells the client how
long to wait before it reconnects; `s.close()` ends the stream, and the client
reconnects. Each write fails with kind `Closed`, `TimedOut` or `Net` once the
client has gone away; stop feeding the stream then. *(ADR TBD)*

**std.net.sse.9** — `s.pipe(ch Channel[Event], heartbeat_ms int) ! SseError`
sends every event received from `ch` until `ch` is closed and empty, then
returns. When `heartbeat_ms` is above 0, a comment is sent each time that many
milliseconds pass without an event. A write that fails closes the stream and
fails the call with its error. `pipe` is how a task feeds many streams: each
stream's task calls `pipe` on its own channel, and a producer sends to those
channels. *(ADR TBD)*

**std.net.sse.10** — `srv.run(handler fn(Request)) ! Trapped` answers clients
until `srv.close()`: 64 tasks each accept one request at a time and call
`handler`, which answers with `req.stream()`, `req.reject(...)` or
`req.close()`; the connection is closed when the handler returns, so 64 streams
can be open at once. `srv.run_with(handler, workers int)` sets the number. The
handler picks the stream by `req.path`; the `std.net.http` `Router` cannot
stream (its handlers return one whole response), so a program that serves both
listens on two addresses, or answers `GET /events` with `sse.response`. *(ADR TBD)*

```ail norun rules=std.net.sse.6,std.net.sse.7,std.net.sse.8,std.net.sse.9,std.net.sse.10
use std.net.sse

fn handle(req sse.Request)
    if req.path != "/events"
        req.reject(404, "not found")
        return
    s := req.stream() else err
        return
    ch := Channel[sse.Event](8)
    for n in 0..3
        ch.send(sse.named("tick", "{n}").with_id("{n}"))
    ch.close()
    s.pipe(ch, 15000) else err
        return

fn main() ! sse.SseError | Trapped
    srv := sse.listen("127.0.0.1:8080")?
    srv.run(handle)?
```

## Failures

**std.net.sse.11** — Every failure is an `SseError { kind SseKind, message
str }` (`SseKind` is `BadUrl`, `Handshake`, `BadRequest`, `Closed`, `TimedOut`
or `Net`), the kinds as `std.net.1`'s `NetKind`: a `net.NetError` of kind
`Closed` or `TimedOut` keeps its kind and every other transport failure is
`Net`. The module is native only and no part of it is Unix-only. *(ADR TBD)*

```ail rules=std.net.sse.11
use std.net.sse

fn main()
    _ := sse.connect("ftp://example.com/") else err
        print("{err.kind}")   # BadUrl
```
