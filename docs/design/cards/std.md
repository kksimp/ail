# std

## (root)
```
error struct IoError  ## std.fs
    path str
    kind str
struct Metadata  ## std.fs.10 (ADR 0281)
    size int
    modified_ms int
    is_dir bool
struct Output  ## std.proc
    status int
    out str
    err str
error struct ProcError
    cmd str
    kind str
struct Response  ## std.net (ADR 0283)
    status int
    headers Map[str, str]
    body str
error struct NetError
    url str
    kind str
    message str
struct Options  ## std.net.6: what a request can set beyond its method and URL.
    headers Map[str, str]
    query Map[str, str]
    body str
    timeout_ms int
    max_redirects int
struct Request  ## std.net.9-11: a request the server received, and the server.
    method str
    path str
    query Map[str, str]
    headers Map[str, str]
    body str
    peer str
struct Server
    handle int
struct Exchange
    request net.Request
    handle int
struct Listener  ## std.net.12: raw TCP.
    handle int
struct Conn
    handle int
struct Window  ## std.ui (ADR 0177). A window is a handle; its cells live in the bootstrap.
    handle int
struct Size
    cols int
    rows int
struct Modifiers
    ctrl bool
    alt bool
    shift bool
enum Event
    Key(key str, ctrl bool, alt bool, shift bool)
    Resize(cols int, rows int)
    Close
    Mouse(col int, row int, button str, kind str)
    Scroll(lines int)
struct Style  ## std.ui.16: how `put_styled` draws text.
    bold bool
    italic bool
    underline bool
enum Cursor  ## std.ui.18: the caret's shape.
    Bar
    Block
    Underline
error struct UiError
    kind str
    message str
struct Tensor[T]  ## std.tensor (ADR 0328): opaque; build with `tensor.from(xs, shape)` or
    elements Map[int, T]
```
