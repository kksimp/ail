# Standard library: attach and `@observe`

Chapter key: `attach`

> **Status:** Drafted (ADR 0884, amending ADR 0879). The field attribute, its
> check, the protocol functions, the listener and the app wrapper are built;
> generating `observe` impls and the `ail run` default are not yet.

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
none. With attach on, the compiler writes the `impl` of a type with `@observe`
fields (not built yet); a type may also write it by hand. With attach off
nothing is generated. *(ADR 0884)*

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

```ail skip=bootstrap rules=attach.wire.1,attach.wire.6
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
