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
error struct CompressError  ## std.compress (ADR 0443)
    kind str
    message str
struct Output  ## std.proc
    status int
    out str
    err str
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
error struct NetError
    url str
    kind str
    message str
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
