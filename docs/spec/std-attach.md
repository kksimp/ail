# Standard library: attach and `@observe`

Chapter key: `attach`

> **Status:** Drafted (ADR 0884, amending ADR 0879). The field attribute, its
> check, the protocol functions, the listener and the app wrapper are built;
> the `Observe` generator is built; the `ail run` default and `ail build --attach`
> (a source shim, `attach.run.1-3`) are built.

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

**attach.wire.4** — The queries are `observe`, `screen` and `heap`, and `input`
(`attach.input.1`). Any other `q` gets an error reply that says the queries are
those and that all but `input` only read. `screen` answers with the `std.screen` query
named by `x`, `y`, `w`, `h`, `role`, `color`, `text` and `budget`. `heap` gives
the summary the program supplies, or an error when it has none. `input`
(`attach.input.1-3`) is the one request that acts; the others only read.
*(ADR 0884, ADR 0919)*

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
inbox, input)` the second, with the host program `cmd` and the `inbox` as
`app.run_with` takes them; `serve.run_with(mut app, cmd, args, inbox)` is both.
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
call and each `app.run_with(` call a `std.attach.serve.run_with(` call (the same
arguments: the host program, its arguments and the program's inbox), and an
`impl attach.Observe` added for each non-generic
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

## The client

**attach.cli.1** — `ail attach` with no argument prints `127.0.0.1:PORT` for each
program whose discovery file is in the directory of `attach.wire.1` (`$TMPDIR`, else
`/tmp`, or `%TEMP%` on Windows), one per line; exit status 1 with a message when there is none. `ail attach
[--port N] observe|screen|heap|input [key=value ...]` sends that query to the program,
with the token from its discovery file, and prints the reply line. The pairs are
the request's members: `x`, `y`, `w`, `h`, `dx`, `dy`, `budget` and `since` are numbers,
`ctrl`, `alt`, `shift` and `meta` are `true` or `false`,
`names` is a comma-separated list, anything else is a string (`role`, `color`,
`text`, `kind`, `id`). *(ADR 0884, ADR 1045)*

**attach.cli.2** — With several programs published, `--port N` names one; without it
the command stops with exit status 2 and says so. A connection or protocol error is
exit status 2.

**attach.cli.3** — The exit status is 1 when the reply is an error reply (it has an
`"error"` member), otherwise 0. The client sends no query but those of
`attach.wire.4`; only `input` acts (`attach.input.1`).

```ail fragment rules=attach.cli.1,attach.cli.2,attach.cli.3
# ail attach                              127.0.0.1:41873
# ail attach screen role=button budget=200
# ail attach observe names=n,total since=3
```
