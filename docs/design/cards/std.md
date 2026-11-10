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
struct Volume  ## std.fs.24 (ADR 0928)
    total int
    free int
error struct CompressError  ## std.compress (ADR 0443)
    kind str
    message str
struct Entry  ## std.fs.23 (ADR 0927): one entry of `fs.scan`, and the scan.
    path str
    size int
    alloc int
    modified_ms int
    inode int
    device int
    links int
    is_dir bool
    is_link bool
struct Scan
    entries List[fs.Entry]
    skipped List[str]
struct ScanCount  ## std.fs.27 (ADR 0992): what every `fs.scan` has listed so far.
    entries int
    bytes int
struct Output  ## std.proc
    status int
    out str
    err str
struct RawOutput  ## std.proc.8 (ADR 0417): the output of `run_bytes`.
    status int
    out bytes
    err bytes
error struct ProcError
    cmd str
    kind str
unique struct Child  ## std.proc.4 (ADR 0424): a child; `stdin` takes `send` and `close`, `out` and `err` give lines.
    pid int
    stdin Channel[str]
    out Channel[str]
    err Channel[str]
struct Response  ## std.net (ADR 0283)
    status int
    headers Map[str, str]
    body str
    all_headers Map[str, List[str]]
struct RawResponse  ## std.net.16 (ADR 0416): a response with its body undecoded.
    status int
    headers Map[str, str]
    data bytes
    all_headers Map[str, List[str]]
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
    data bytes
    connect_ms int
    total_ms int
struct Request  ## std.net.9-11: a request the server received, and the server.
    method str
    path str
    query Map[str, str]
    headers Map[str, str]
    body str
    peer str
    data bytes
    server int
struct Body  ## std.net.25: a response whose body is read in pieces.
    status int
    headers Map[str, str]
    all_headers Map[str, List[str]]
    handle int
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
struct Regex  ## std.regex (ADR 0438). A regex is its pattern; compiled programs are cached
    pat str
error struct RegexError
    pattern str
    at int
    kind str
    rule str
    message str
struct Match
    start int
    end int
    text str
struct Captures  ## `groups[0]` is the whole match; a group that didn't take part is `none`.
    start int
    end int
    groups List[str?]
    named Map[str, str]
```
