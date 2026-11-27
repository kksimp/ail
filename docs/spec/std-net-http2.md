# Standard library: `std.net.http2`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD); native only.

An HTTP/2 server (RFC 9113) with its HPACK coder (RFC 7541), written in AIL
over `net.Conn`. It answers the same handler as `std.net.http`,
`fn(net.Request) net.Response`. The module is a part of `std.net` and needs
`use std.net.http2` (`std.use.4`); it is used as `http2`. The client side of
HTTP/2 is `std.net.31`: `net.get` and the other client calls already speak
`h2` over TLS.

## The server

**std.net.http2.1** — `serve_conn(conn net.Conn, handler fn(net.Request)
net.Response) ! H2Error` speaks HTTP/2 on a connection that is already open:
plain TCP whose client starts with the connection preface (h2c with prior
knowledge), or TLS whose ALPN choice was `h2` (`std.net.46`). It reads the
preface, sends its SETTINGS (MAX_CONCURRENT_STREAMS 100, INITIAL_WINDOW_SIZE
1048576, MAX_FRAME_SIZE 16384, MAX_HEADER_LIST_SIZE 65536) and a
WINDOW_UPDATE that opens the connection's receive window to 16 MiB, and
requires the client's first frame to be SETTINGS. It returns when the client
closes the connection, sends GOAWAY, or sends nothing for the connection's
timeout (`Conn.set_timeout`, 30 s by default) between frames; it then sends
GOAWAY with `NO_ERROR` and the last stream it processed, and closes the
connection. A client that sends anything but the preface fails with kind
`NotHttp2`; one whose first bytes are not `PRI ` (an HTTP/1.1 request) is first
answered `505 HTTP Version Not Supported`. *(ADR TBD)*

**std.net.http2.2** — Streams are multiplexed. A request is complete at the
END_STREAM flag of its HEADERS or of its last DATA frame. When the server has
handled every whole frame the client has sent so far, it answers all the
complete requests at once, each `handler` call in its own task, and sends the
responses' HEADERS in the order the requests completed and their DATA frames
interleaved, one frame per stream in turn. While every stream with body left
has a shut window, it reads and handles the client's frames, so requests that
arrive then are answered in the next round. A handler that traps traps the
connection's task (`err.trap.1`), which `run` passes on as `Trapped`. *(ADR TBD)*

**std.net.http2.3** — A request whose fields break RFC 9113 8.2-8.3 is
refused with RST_STREAM `PROTOCOL_ERROR` and never reaches the handler: a
field name with an upper-case letter or empty, a pseudo-header field other
than `:method`, `:scheme`, `:path` and `:authority`, a repeated one, one after
a regular field, a missing `:method`, `:scheme` or `:path` (so CONNECT is
refused), a connection-specific field (`connection`, `keep-alive`,
`proxy-connection`, `transfer-encoding`, `upgrade`), `te` other than
`trailers`, or a `content-length` that differs from the body's length. A
stream past the 100 that are open is refused with `REFUSED_STREAM`. A header
block over 64 KiB ends the connection with `ENHANCE_YOUR_CALM`. A body over 16
MiB is answered `413` without the handler. Trailers (a second HEADERS that
ends the stream) end the request; their fields are not kept. *(ADR TBD)*

**std.net.http2.4** — The handler gets the `net.Request` of `std.net.9`:
`method` is `:method`; `path` is `:path` before `?` and `query` the rest,
split and decoded as `std.net.10` does; `headers` has every regular field,
repeated ones joined with `, ` (`cookie` with `; `, RFC 9113 8.2.3), and a
`host` from `:authority` when the request has none; `body` and `data` are the
body as `std.net.16` says; `peer` is the client's address; `server` is `-1`,
so `req.stop()` does nothing (`http.serve.1`): stop a server with
`l.close()` (`std.net.http2.6`). *(ADR TBD)*

**std.net.http2.5** — The response is sent as `std.net.11` sends one, as
HTTP/2 fields. Its status must be 200 to 599 and its header names tokens with
values free of CR, LF and NUL; otherwise the stream is answered `500` with a
plain-text body. Names are sent in lower case; connection-specific fields and
`content-length` in `headers` are dropped, `all_headers` names are sent once
per value in place of their `headers` entry (`std.net.17`), a body without a
`content-type` gets `text/plain; charset=utf-8`, and `content-length` is the
body's length. `204` and `304` have no body, and the answer to `HEAD` has the
headers a `GET` would have and no DATA. *(ADR TBD)*

**std.net.http2.6** — `run(l net.Listener, handler fn(net.Request)
net.Response) ! Trapped` serves every connection `l` accepts with
`serve_conn`, up to 64 at once (64 tasks, each one connection at a time),
until `l.close()`; connections still open are served until they end
(`std.net.http2.1`), and a connection that fails is not the server's failure.
`l` is `net.listen(addr)` for h2c, or `net.listen_tls_alpn(addr, cert, key,
["h2"])` for HTTP/2 over TLS (`std.net.46`). `serve(addr str, handler) !
net.NetError | Trapped` is `run(net.listen(addr)?, handler)`. *(ADR TBD)*

```ail norun rules=std.net.http2.1,std.net.http2.2,std.net.http2.3,std.net.http2.4,std.net.http2.5,std.net.http2.6
use std.net.http2

fn hello(req net.Request) net.Response
    who := req.query.get("name") else "world"
    net.Response(status: 200, headers: {}, body: "hello {who}\n")

fn main() !
    # h2c: `curl --http2-prior-knowledge http://127.0.0.1:8080/?name=ail`
    http2.serve("127.0.0.1:8080", hello)?
```

## Flow control and control frames

**std.net.http2.7** — Flow control (RFC 9113 6.9). The server credits the
connection and the stream with a WINDOW_UPDATE for every DATA frame as it
arrives (the stream only while it is open), so a client is held back only by
the 1048576-byte stream window. It sends no DATA frame larger than the
client's MAX_FRAME_SIZE or than the stream's or the connection's send window,
both starting at 65535 and the stream's following the client's
INITIAL_WINDOW_SIZE, including changes made while streams are open. A window
pushed past 2^31-1 is `FLOW_CONTROL_ERROR`: for the connection or a SETTINGS
change, a connection error; for one stream, RST_STREAM. A WINDOW_UPDATE of 0
is `PROTOCOL_ERROR`, for the connection or the stream it names. *(ADR TBD)*

**std.net.http2.8** — Control frames. SETTINGS is acknowledged; one on a
stream, an ACK with a payload, a length that is not a multiple of 6,
ENABLE_PUSH other than 0 or 1, or MAX_FRAME_SIZE outside 16384 to 16777215 is
a connection error. PING is answered with an ACK and the same 8 bytes. A
client's GOAWAY lets the requests already complete be answered and ends the
connection. PRIORITY is ignored. RST_STREAM drops the stream and any of its
response not yet sent. A frame over 16384 bytes, PUSH_PROMISE, a frame other
than CONTINUATION inside a header block, DATA or HEADERS on stream 0 or on a
stream the client never opened, HEADERS on an even or closed stream, and
RST_STREAM of the wrong length, on stream 0 or on a stream never opened, or
PRIORITY on stream 0 are connection errors; PRIORITY of the wrong length
resets its stream with `FRAME_SIZE_ERROR`, and DATA on a stream that was
reset or answered is dropped. Frames of
unknown types are ignored (RFC 9113 4.1). The server never sends
PUSH_PROMISE. *(ADR TBD)*

**std.net.http2.9** — A connection error (RFC 9113 5.4.1) sends GOAWAY with
the error code, the last stream processed and the reason as debug data,
closes the connection, and fails with kind `Protocol`. `H2Error` has `kind
H2Kind` and `message str`; `H2Kind` is `NotHttp2`, `Protocol`, `Closed` (the
connection ended inside a frame or a response), `TimedOut` (no progress inside
a frame), `Net` or `Usage` (`std.net.http2.13`). The functions over a connection have the effects `net` and
`io`; `serve` also `net.serve` (`std.net.14`). *(ADR TBD)*

## HPACK

**std.net.http2.10** — `decoder() Hpack` makes an HPACK decoder whose dynamic
table is empty with the default size of 4096 bytes; `d.decode(block bytes)
List[Field] ! H2Error`, on a `mut` decoder, decodes one header block as RFC
7541 says (indexed fields, literals with and without indexing and never
indexed, Huffman-coded strings, dynamic table size updates up to 4096) and
keeps the table for the next block of the same connection. `Field` has `name
str` and `value str`. A bad index, integer, string or Huffman code (padding
over 7 bits, or the EOS symbol) fails with kind `Protocol`, whose message
starts `HPACK: `. *(ADR TBD)*

**std.net.http2.11** — `encode(fields List[Field]) bytes` is the header block
of `fields` in order: a field equal to a static table entry is that entry's
index, and every other field is a literal without indexing whose name is the
static table's index when the table has the name; strings are not Huffman
coded. The block never adds to a decoder's dynamic table, so any decoder can
read it. *(ADR TBD)*

```ail tool=run,FILE expect=4,:method=GET,:path=/,:authority=www.example.com,cache-control=no-cache,880f100a746578742f706c61696e,:status=200,content-type=text/plain rules=std.net.http2.10,std.net.http2.11
use std.net.http2

fn main() !
    mut d := http2.decoder()
    # RFC 7541 C.4.1 and C.4.2: the second block uses the first one's table.
    first := d.decode(bytes.from_hex("828684418cf1e3c2e5f23a6ba0ab90f4ff") else b"")?
    print(first.len())
    for f in d.decode(bytes.from_hex("828684be5886a8eb10649cbf") else b"")?
        if f.name != ":scheme"
            print("{f.name}={f.value}")
    block := http2.encode([
        http2.Field(name: ":status", value: "200"),
        http2.Field(name: "content-type", value: "text/plain"),
    ])
    print(block.hex())
    mut e := http2.decoder()
    for g in e.decode(block)?
        print("{g.name}={g.value}")
```

## Streams

**std.net.http2.12** — `serve_streams(conn net.Conn, handler fn(Stream)) !
H2Error` serves a connection as `serve_conn` does (`std.net.http2.1`), but gives
each request stream to `handler` as soon as its headers arrive, one stream at
a time in the order the streams opened; frames of the other streams are
handled and their bodies buffered meanwhile. A `Stream` has `id int`, `method
str`, `path str`, `query Map[str, str]`, `headers Map[str, str]` and `peer str`
as `std.net.http2.4`, and `fields List[Field]`, every field of the request's
HEADERS. `s.read() bytes ! H2Error` returns the body's next DATA frame as it
arrives, empty at the end of the request; `s.read_all()` the rest of it. A
stream that buffers more than 16 MiB unread is reset with
`ENHANCE_YOUR_CALM`. *(ADR TBD)*

**std.net.http2.13** — `s.respond(status int, headers Map[str, str])` sends the
response's HEADERS (status 200 to 599; names in lower case;
`content-length`, pseudo-header and connection-specific fields dropped).
`s.write(data bytes)` sends DATA frames, waiting while the stream's or the
connection's send window is shut and handling the client's frames
meanwhile (`std.net.http2.7`); `s.window() int` is how many bytes it can send
without waiting. `s.finish(trailers Map[str, str])` ends the response with the
trailers as a last HEADERS frame, or an empty DATA frame when there are none.
`s.reset(code int)` sends RST_STREAM with `code`. Each fails with kind `Usage`
when called out of order (`write` or `finish` before `respond`, `respond`
twice) or given a bad status or header, and with kind `Closed` once the stream
was reset or finished. *(ADR TBD)*

**std.net.http2.14** — When `handler` returns, a stream it responded on and
did not finish is finished with no trailers, and one it never responded on is
answered `500` with a plain-text body. A handler that traps traps the
connection's task. A gRPC method is a handler that reads messages with `read`,
answers with `respond(200, {"content-type": "application/grpc"})`, `write` and
`finish({"grpc-status": "0"})`. *(ADR TBD)*

```ail norun rules=std.net.http2.12,std.net.http2.13,std.net.http2.14
use std.net.http2

fn echo(s http2.Stream)
    s.respond(200, {"content-type": "application/octet-stream"}) else err
        return
    while true
        chunk := s.read() else err
            return
        if chunk.len() == 0
            break
        s.write(chunk) else err
            return
    s.finish({"x-done": "yes"}) else err
        return

fn main() !
    l := net.listen("127.0.0.1:8080")?
    conn := l.accept()?
    http2.serve_streams(conn, echo)?
```

The tests are `tests/native/run/net_http2.ail` (an h2c client over
`net.Conn`: two streams on one connection, a body, PING, GOAWAY for a
protocol error, `505` for HTTP/1.1, `l.close()`) and
`tests/native/run/net_http2_hpack.ail` (RFC 7541 C.3 and C.4, `encode`, bad
blocks) and `tests/native/run/net_http2_streams.ail` (`serve_streams`: a body
echoed frame by frame with trailers, a handler that never responds, a reset).
