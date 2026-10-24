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

```ail skip=bootstrap rules=attach.observe.2
struct Cart
    @observe total int?
    name str

fn main()
    c := Cart(total: none, name: "tea")
    print(c.name)
```

```ail skip=bootstrap rules=attach.observe.1
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

**attach.wire.4** — The queries are `observe`, `screen` and `heap`. Any other
`q`, including one that asks to set or change something, gets an error reply
that says all queries only read. `screen` answers with the `std.screen` query
named by `x`, `y`, `w`, `h`, `role`, `color`, `text` and `budget`. `heap` gives
the summary the program supplies, or an error when it has none. *(ADR 0884)*

**attach.wire.5** — `observe` answers `{"v":1,"rev":N,"values":{name:value}}`.
`names` limits the values; `since` limits them to those that changed after
revision `since`. A `Tracker` raises its revision once per `update` that adds
a value or changes one. *(ADR 0884)*

```ail skip=bootstrap rules=attach.wire.2,attach.wire.3,attach.wire.4,attach.wire.5
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
`$TMPDIR/ail-attach-PORT.json` (`/tmp` without `TMPDIR`), created empty,
restricted to mode 0600, then filled; the file is removed when the window
closes. The token is 24 random bytes in hex. Nothing listens until `run` is
called. *(ADR 0884)*

**attach.wire.6** — The listener thread never reads program state. It posts
each request line to the app's inbox as an `external` event from `attach`,
and the app answers between its other events with the observed values (the
`Observe` trait), the screen of its view tree (`std.screen.view`) and the
protocol reply. A client that sends nothing for 5 seconds is dropped. *(ADR 0884)*

```ail skip=bootstrap rules=attach.observe.4,attach.wire.1,attach.wire.6
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
`impl app.App for T` that has none. A program that doesn't run an app is built
as written. *(ADR 0884)*

**attach.run.2** — If the rewritten program doesn't build, the build warns
`attach is off for this run` with the reason and builds the program as written.
Attach off adds no listener, no thread and no `observed` method (zero cost).
*(ADR 0884)*

**attach.run.3** — The cache key of a build includes whether attach is on.
*(ADR 0884)*

```ail skip=bootstrap rules=attach.run.1,attach.run.2,attach.run.3
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
