# Standard library: attach and `@observe`

Chapter key: `attach`

> **Status:** Drafted (ADR 0884, amending ADR 0879; ADR 0986 adds `@tool`). The field attribute, its
> check, the protocol functions, the listener and the app wrapper are built;
> the `Observe` generator is built; the `ail run` default and `ail build --attach`
> (a source shim, `attach.run.1-3`) are built; `@tool` methods, `tools`, `call`
> and `ail attach mcp` (`attach.tool.1-5`) are built.

A running program tells an attached agent its state directly, read-only.

**attach.observe.1** — An `@observe` field has type `int`, `float`, `bool` or
`str`, a payload-less enum, or a `List` of those. Anything else cannot be read
as text. *(ADR 0884)*

**attach.observe.2** — An `@observe` field of any other type is a compile-time
error whose `help` lists the allowed types. *(ADR 0884)*

```ail error=attach.observe.2 rules=attach.observe.2
struct Cart
    @observe total int?
    name str

fn main()
    c := Cart(total: none, name: "tea")
    print(c.name)
```

```ail rules=attach.observe.1
enum Mode
    On
    Off

struct Panel
    @observe count int
    @observe tags List[str]
    @observe mode Mode

fn main()
    p := Panel(count: 1, tags: ["a"], mode: Mode.On)
    print(p.count)
```

**attach.observe.4** — `attach.Observe` is the trait of types that report
their observed values: `observed(self) List[Observed]`, whose default reports
none. An `impl attach.Observe for T` with no methods gets `observed` from a
built-in generator (as `ToJson` does, ADR 0850): one `Observed` per `@observe`
field, in order, with the field's name, its type as written and its value as
JSON text (numbers and bools as written, a `str` quoted, a payload-less enum
as its variant name in quotes, a `List` as a JSON array). A type may write the
`impl` by hand instead. Without such an `impl`, nothing is generated. *(ADR 0884)*

## The protocol

`use std.attach` provides the protocol as pure functions: `Observed`,
`Tracker`, `answer`, `authorized`. The listener that carries it is separate.

**attach.wire.2** — A request line without the token the program published, or
with another one, is answered with an error and does nothing. *(ADR 0884)*

**attach.wire.3** — A request is one JSON object on a line and a reply is one
JSON object on a line, with `"v":1`. A line that is not JSON gets an error
reply. *(ADR 0884)*

**attach.wire.4** — The queries are `observe`, `screen`, `heap` and `tools`, and
`input` and `call` (`attach.input.1`, `attach.tool.4`). Any other `q` gets an error reply that says the queries are
those and that all but `input` and `call` only read. `screen` answers with the `std.screen` query
named by `x`, `y`, `w`, `h`, `role`, `color`, `text` and `budget`. `heap` gives
the summary the program supplies, or an error when it has none. `input`
(`attach.input.1-3`) and `call` (`attach.tool.4`) are the requests that act; the
others, `tools` among them, only read. *(ADR 0884, ADR 0919, ADR 0986)*

**attach.wire.5** — `observe` answers `{"v":1,"rev":N,"values":{name:value}}`.
`names` limits the values; `since` limits them to those that changed after
revision `since`. A `Tracker` raises its revision once per `update` that adds
a value or changes one. *(ADR 0884)*

```ail rules=attach.wire.2,attach.wire.3,attach.wire.4,attach.wire.5
use std.attach
use std.screen

fn main()
    mut tr := attach.Tracker.new()
    tr.update([attach.observed("total", "int", "3")])
    s := screen.Screen(w: 10.0, h: 10.0, items: [])
    print(attach.answer("\{\"q\":\"observe\",\"token\":\"k\"}", "k", tr, s, ""))
    print(attach.answer("\{\"q\":\"observe\"}", "k", tr, s, ""))
```

## The listener

**attach.wire.1** — `attach.serve.run(mut app)` listens on `127.0.0.1` only,
on a free port, and publishes `{"addr":"127.0.0.1:PORT","token":"..."}` in
`ail-attach-PORT.json`, created empty, restricted to mode 0600, then filled.
The file's directory is `$TMPDIR`; without `TMPDIR` it is `/tmp`, or on Windows
`%TEMP%`, the temporary folder inside the user's profile, whose access list
takes the place of the mode. The token is 24 random bytes in hex. Nothing
listens until `run` is called. `serve.open()` is the first half of `run` (the
listener and its file, as a `Served`) and `serve.run_on(s, mut app, cmd, args,
input)` the second, with the host program `cmd` as `app.run_with` takes it.
*(ADR 0884, ADR 1045)*

**attach.wire.6** — The listener thread never reads program state. It posts
each request line to the app's inbox as an `external` event from `attach`,
and the app answers between its other events with the observed values (the
`Observe` trait), the screen of its view tree (`std.screen.view`) and the
protocol reply. A client that sends nothing for 5 seconds is dropped. *(ADR 0884)*

**attach.wire.7** — When the window closes, or its host can't start, the
listener stops before `run` returns: a request not yet answered gets an empty
line, the connection in progress is closed after its reply, the socket is
closed and the discovery file is removed. This holds on Linux, macOS and
Windows. *(ADR 1045)*

```ail norun rules=attach.observe.4,attach.wire.1,attach.wire.6,attach.wire.7
use std.app
use std.attach
use std.attach.serve

struct Shop
    n int

impl app.App for Shop
    fn update(mut self, ev app.Event)
        n += 1

    fn view(self) app.Node
        app.label("hi")

impl attach.Observe for Shop
    fn observed(self) List[attach.Observed]
        [attach.observed("n", "int", "{n}")]

fn main() !
    mut s := Shop(n: 0)
    serve.run(mut s)?
```

**attach.run.1** — `ail run FILE` builds a program that calls `app.run(` with
attach on, unless it is given `--no-attach` before the file (the `ail` script passes it to `prepare`, which attaches by default). `ail build
--attach` does the same for a built executable; without `--attach`, `ail build`
adds nothing. The build rewrites the source text before it is lowered: two
imports on new first lines, each `app.run(` call made a `std.attach.serve.run(`
call, and an `impl attach.Observe` added for each non-generic
`impl app.App for T` that has none. A program that doesn't run an app, or that already uses the name `attach`
or `ail_serve` for another module, is built as written. *(ADR 0884)*

**attach.run.2** — If the rewritten program doesn't build, the build warns
`attach is off for this run` with the reason and builds the program as written.
Attach off adds no listener, no thread and no `observed` method (zero cost).
*(ADR 0884)*

**attach.run.3** — The cache key of a build includes whether attach is on.
*(ADR 0884)*

**attach.run.4** — A program that imports `std.gfx.host` on a line of its own
and calls `host.run(` is rewritten the same way: each `host.run(` call becomes a
`std.gfx.watch.run(` call and an `impl attach.Observe` is added for each
non-generic `impl gfx.Game for T` that has none, so `ail attach screen` and
`observe` work on a game's window (`std.gfx.10`). *(ADR 0979)*

```ail norun rules=attach.run.1,attach.run.2,attach.run.3
# ail run counter.ail            attach on: the socket is served
# ail run --no-attach counter.ail   attach off
# ail build --attach counter.ail -o counter
use std.app

struct Counter
    @observe n int

impl app.App for Counter
    fn update(mut self, ev app.Event)
        n += 1

    fn view(self) app.Node
        app.label("{n}")

fn main() !
    mut c := Counter(n: 0)
    app.run(mut c)?
```

## Input

**attach.input.1** — A program accepts `input` requests only when it was started
with `ail run --attach-input` (or with `AIL_ATTACH_INPUT=1` in its environment,
which is what the flag sets). Otherwise `input` gets an error reply that says to
start the program with `--attach-input`, and nothing changes. The flag is off
by default; a program started without it can never be driven. The token (`attach.wire.2`) and the 127.0.0.1-only listener
(`attach.wire.1`) apply as for every request. *(ADR 0919)*

**attach.input.2** — An `input` request is `{"v":1,"q":"input","token":T,"kind":K,...}`
whose other members are the fields of a `std.app` `Event` (`std.app.5`: `id`,
`text`, `x`, `y`, `dx`, `dy`, `ctrl`, `alt`, `shift`, `meta`). The program
handles the event between its own events, through its `update`, exactly as it
would one from the window. `id` is a control's own id (`Button("pay", ...)` is
`pay`) or the `#N` that `screen` shows for it; an id that names no control of
the view gets an error reply and nothing changes. A `value` member is an error:
a field's new text is `text`. The reply is `{"v":1,"applied":K}`; the agent reads the
result with `observe` or `screen`. *(ADR 0919)*

**attach.input.3** — `kind` is one of `press`, `edit`, `submit`, `toggle`,
`click`, `scroll` and `key`: what a person does at the window. Any other kind
(`start`, `resize`, `external`, or none) gets an error reply and changes nothing.
*(ADR 0919)*

```ail fragment rules=attach.input.1,attach.input.2,attach.input.3
# ail run --attach-input shop.ail
# ail attach input kind=press id=pay
# ail attach input kind=edit id=name text=Ada
# ail attach input kind=click id=board x=40 y=12
# ail attach screen role=button
```

## Tools

An app offers actions in its own terms (`add(item, qty)`), so an agent need not
find and press the controls that do them.

**attach.tool.1** — `@tool` before a method `fn T.name(self or mut self, ...)` of a
non-generic struct `T` offers it to an attached agent. Its parameters are `int`,
`float`, `bool`, `str` or an enum without payloads; its result is nothing or a
type `attach.observe.1` allows, and it may fail (`!`). Its `##` doc describes it.
Without attach the method is an ordinary method. *(ADR 0986)*

**attach.tool.2** — Any other `@tool` function (a free function, one without
`self`, a generic one, or one with another parameter or result type) is a
compile-time error whose `help` gives the allowed shape. *(ADR 0986)*

```ail error=attach.tool.2 rules=attach.tool.2
struct Shop
    n int

@tool fn Shop.add_all(mut self, xs List[int])
    n += xs.len()

fn main()
    print(Shop(n: 0).n)
```

**attach.tool.3** — The generated `attach.Observe` (`attach.observe.4`) also
writes `tools(self) List[attach.Tool]`, one `Tool` per `@tool` method of `T` in
order (name, doc, parameter names and types, result type or `""`), and
`call(mut self, name, args)`. The defaults offer none. A `tools` request needs
only the token and answers `{"v":1,"tools":[{"name":N,"doc":D,"params":[{"name":P,"type":T}],"result":R}]}`.
*(ADR 0986)*

**attach.tool.4** — A `call` request is `{"v":1,"q":"call","token":T,"name":N,"args":{...}}`.
Like `input`, it is applied only when the program was started with
`--attach-input` (`attach.input.1`), else it gets an error reply and nothing runs.
The app runs the method between its events and replies `{"v":1,"result":JSON}`
(`null` for no result). Each argument is read by its parameter's type: a JSON
value, or text of it (`"2"` for an `int`, `"true"`, a variant's name); a missing
or wrong one, an unknown `name`, or a failure of the method gets an error reply
naming it. *(ADR 0986)*

```ail rules=attach.tool.1,attach.tool.3,attach.tool.4
use std.attach

struct Cart
    @observe total int

## Adds `qty` items; gives the new total.
@tool fn Cart.add(mut self, qty int) int
    total += qty
    total

impl attach.Observe for Cart

fn main()
    mut c := Cart(total: 0)
    print(c.tools()[0].doc)
    args := parse_json("\{\"qty\":\"2\"}") else Json.Obj([])
    print(c.call("add", args))
```

**attach.tool.5** — `ail attach [--port N] mcp` is an MCP server on stdio
(`initialize`, `ping`, `tools/list`, `tools/call`, as `ail mcp`) for the running
program: its tools are `app_observe`, `app_screen` and `app_input` (the queries)
and each of the program's `@tool` methods, with an input schema from the
parameter types (`int` an `integer`, `float` a `number`, `bool` a `boolean`,
others a `string`). A call is one attach request; its reply line is the result
text, an error result when it has `error`. *(ADR 0986)*

```ail fragment rules=attach.tool.5
# ail run --attach-input shop.ail
# ail attach tools
# ail attach call add qty=2
# claude mcp add shop -- ail attach mcp
```

## The client

**attach.cli.1** — `ail attach` with no argument prints `127.0.0.1:PORT` for each
program whose discovery file is in the directory of `attach.wire.1` (`$TMPDIR`, else
`/tmp`, or `%TEMP%` on Windows), one per line; exit status 1 with a message when there is none. `ail attach
[--port N] observe|screen|heap|tools|input [key=value ...]` sends that query to the program,
with the token from its discovery file, and prints the reply line. The pairs are
the request's members: `x`, `y`, `w`, `h`, `dx`, `dy`, `budget` and `since` are numbers,
`ctrl`, `alt`, `shift` and `meta` are `true` or `false`,
`names` is a comma-separated list, anything else is a string (`role`, `color`,
`text`, `kind`, `id`). `ail attach call TOOL [key=value ...]` sends a `call` whose
pairs are the arguments, each as text (`attach.tool.4`). *(ADR 0884, ADR 0986, ADR 1045)*

**attach.cli.2** — With several programs published, `--port N` names one; without it
the command stops with exit status 2 and says so. A connection or protocol error is
exit status 2.

**attach.cli.3** — The exit status is 1 when the reply is an error reply (it has an
`"error"` member), otherwise 0. The client sends no query but those of
`attach.wire.4`; only `input` and `call` act (`attach.input.1`, `attach.tool.4`).

```ail fragment rules=attach.cli.1,attach.cli.2,attach.cli.3
# ail attach                              127.0.0.1:41873
# ail attach screen role=button budget=200
# ail attach observe names=n,total since=3
```
