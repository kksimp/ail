# Standard library: attach and `@observe`

Chapter key: `attach`

> **Status:** Drafted (ADR 0884, amending ADR 0879). The field attribute and its
> check are built; the attach server and the `ail run` default are not yet.

A running program tells an attached agent its state directly, read-only.

**attach.observe.1** — An `@observe` field has type `int`, `float`, `bool` or
`str`, a payload-less enum, or a `List` of those. Anything else cannot be read
as text. *(ADR 0884)*

**attach.observe.2** — An `@observe` field of any other type is a compile-time
error whose `help` lists the allowed types. *(ADR 0884)*

```ail error=attach.observe.2
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
