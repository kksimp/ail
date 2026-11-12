# 18. Effects and capabilities

Chapter key: `eff`

> **Status:** Drafted (partial). Specified: inferred effect sets, `@pure`, and
> denying capabilities to a program. Not yet designed: effects written in
> signatures. Capability grants in package manifests are specified with
> denial. Deterministic mode is specified in the last section.

## Inference

**eff.infer.1** — Every function has an *effect set*: the effects `io`,
`fail`, `mutates`, `unsafe`, `hw` and `spawn`, and the *capabilities* `fs`, `env`,
`proc`, `clock`, `random`, `net`, `net.serve`, `ui` and `app`. The set is inferred; it is never written in
source. *(ADR 0163)*

**eff.infer.2** — A function has `fail` if its return type is `T ! E`, `! E`,
`!T` or `!`, and `mutates` if it has a `mut` parameter or `mut self`. Both
come from its own signature only. *(ADR 0163)* A function also has `mutates`
if its body writes through a `ref` (`own.ref.4`): assigns to a place in a
cell, passes one `mut`, or calls a `mut self` method on a `ref`. *(ADR 0192)*

**eff.infer.3** — A function has every effect other than `fail` and
`mutates` that any function it may call has. The effect sets of a program
are the least sets that satisfy this rule (a fixed point over the call
graph, so recursion is allowed). Constants contribute nothing: they are
computed at compile time and may call only compile-time-safe code
(`ct.const.2`). *(ADR 0163, ADR 0225)*

**eff.infer.4** — The sources of effects are: `print` gives `io`; the
standard modules give their module's effects: `fs.*` gives `fs` and `io`,
`env.*` gives `env`, `io.*` gives `io`, `proc.*` gives `proc` and `io`,
`net.*` gives `net` and `io` (`std.net.1`), and `net.serve` and `net.listen` also give `net.serve` (`std.net.14`), `time.*` gives `clock`, `random.*` gives `random`, and `ui.*` and the methods
of `ui.Window` give `ui` and `io` (`std.ui.9`), `app.run_host` (and so `app.run`) gives `app` (`std.app.12`); an `unsafe` block gives `unsafe`; `std.hw` and volatile pointer methods give `hw` (`unsafe.hw.5`); `spawn` gives
`spawn`. *(ADR 0163, ADR 0283, ADR 0838)*

**eff.infer.5** — A call whose target is known from the types (a free
function, an inherent method, an associated function, or a trait method on a
value whose type has an `impl`) may call only that target. Any other trait
call (through `any Trait`, a type parameter, or the qualified form) may call
every implementation of that method in the program, including the trait's
default. Operators other than arithmetic and `and`/`or`, string
interpolation, `print` and the built-in methods may call every implementation
of the standard traits `Eq`, `Ord`, `ToStr` and `Hash`. *(ADR 0163, ADR 0015)*

**eff.infer.6** — `ail effects FILE` prints one line per function, `ID {e1,
e2}`, sorted by stable id (`fn:name`, `method:Type.name`) with each set
sorted. With `--json`, it prints one object
`{"ail_api":1,"effects":{"ID":["e1","e2"],...}}`. The prelude's functions are
not listed. *(ADR 0163)*

```ail
fn log(msg str)
    print(msg)

fn main()
    log("hi")
```

`ail effects` prints `fn:log {io}` and `fn:main {io}`.

## Pure functions

**eff.pure.1** — A function marked `@pure` MUST NOT have `io`, `unsafe`, `hw`,
`spawn` or any capability. It MAY have `fail` and `mutates`. The diagnostic
names, for each forbidden effect, the shortest chain of calls that adds it,
such as `f -> save -> log -> print`. *(ADR 0164)*

```ail
@pure fn double(x int) int
    x * 2

fn main()
    print(double(21))
```

## Denying capabilities

**eff.cap.1** — `ail run --deny LIST FILE` and `ail check --deny LIST FILE`
reject, at compile time, a program whose `main` has a denied capability. The
diagnostic is at `main` and names the chain of calls that reaches the
capability, such as `main -> save -> fs.write_text`. *(ADR 0165)*

**eff.cap.2** — `LIST` is a comma-separated list of capabilities, or `all`
for every capability. Any other name is a usage error (exit status 2) that
lists the capabilities. In a manifest's `caps LIST` (`eff.cap.4–5`), `none`
is also accepted, and an unknown name is a compile-time error at the
manifest line. *(ADR 0165, ADR 0232)*

**eff.cap.3** — `io` and the other effects are not capabilities and can't be
denied: printing is how a program reports, and failure and mutation are
already in signatures. *(ADR 0165)*

**eff.cap.4** — A `dep` line's `caps LIST` (`mod.pkg.2`) grants that
dependency the listed capabilities and no others; a `dep` line without
`caps` grants none. Each function belongs to the package whose name is the
first segment of its module, or else to the root package. A package's
functions, including what they reach through other packages, must reach only
capabilities that each grant of that package includes. Otherwise it is a
compile-time error at the manifest line, one per grant and capability,
naming a chain of calls from a function of the package (a public one if any
reaches the capability), such as `geo.size -> clockwork.home -> env.cwd`.
*(ADR 0232)*

```ail pkg=caps_denied tool=check,DIR status=2 expect=eff.cap.4 rules=eff.cap.4
```

**eff.cap.5** — `caps LIST` on a `package` line limits that package's own
functions the same way (`eff.cap.4`); for the root package it is an
allow-list for the program. A `package` line without `caps` sets no limit.
*(ADR 0232)*

```ail pkg=caps_denied tool=check,DIR status=2 expect=eff.cap.5 rules=eff.cap.5
```

## Deterministic execution

**eff.det.1** — A program whose `main` reaches none of the capabilities
`clock`, `random`, `proc` and `net`, and has no `spawn`, produces the same
output on every run given the same inputs: its arguments, environment
variables, files and standard input. Map and set iteration is in insertion
order (`type.map.1`), and hash values are never visible, so neither can vary
a run. Programs with tasks are outside this rule until task scheduling is
specified. *(ADR 0207)*

```ail rules=eff.det.1
fn main()
    mut counts := Map[str, int]()
    for w in ["b", "a", "b"]
        counts[w] += 1
    for k in counts.keys()           # insertion order: no clock, random, proc or net
        print("{k}: {counts[k]}")
```

**eff.det.2** — `ail run --deterministic FILE` and `ail check
--deterministic FILE` are `--deny clock,net,proc,random` (`eff.cap.1`), and
combine with `--deny`. `fs`, `env` and standard input are inputs, not
sources of nondeterminism, so they stay allowed. *(ADR 0207)*

```ail error=eff.cap.1 flags=--deterministic rules=eff.det.2
use std.time

fn main()
    print(time.now_ms() > 0)         # error: the clock is denied in deterministic mode
```

**eff.det.3** — `ail run --seed S FILE` seeds `std.random` (`std.random.3`).
A program that uses `random` and runs with a seed repeats without being
denied the capability. *(ADR 0207)*

```ail tool=run,--seed,42,FILE expect=true rules=eff.det.3
use std.random

fn main()
    random.seed(1)
    print(random.int(1, 6) <= 6)
```

## Examples

Conformance examples (ADR 0002). `tool=` examples run `ail effects` or
another tool on the block and check its output; `flags=` examples are checked
with those `ail check` flags; `fixture=` examples use a case fixture that
must report the cited rules.

```ail tool=effects,FILE expect=fn:log rules=eff.infer.1,eff.infer.6
fn log(msg str)
    print(msg)

fn main()
    log("hi")
```

```ail tool=effects,FILE expect=fn:bump rules=eff.infer.2
fn bump(mut xs List[int])
    xs.push(1)

fn main()
    mut ys := List[int]()
    bump(mut ys)
    print(ys.len())
```

```ail tool=effects,FILE expect=fn:outer rules=eff.infer.3
fn inner()
    print("deep")

fn outer()
    inner()

fn main()
    outer()
```

```ail tool=effects,--json,FILE expect=ail_api rules=eff.infer.4,eff.infer.6
use std.env

fn home_dir() str
    env.cwd()

fn main()
    print(home_dir().len() > 0)
```

```ail rules=eff.infer.5
trait Speak
    fn say(self) str

struct Dog
    name str

impl Speak for Dog
    fn say(self) str
        "woof from {self.name}"

fn talk(s any Speak) str
    s.say()

fn main()
    print(talk(Dog(name: "rex")))
```

```ail error=eff.pure.1
fn log(msg str)
    print(msg)

@pure fn double(x int) int
    log("doubling")               # error: a @pure function may not do io
    x * 2

fn main()
    print(double(21))
```

```ail flags=--deny,fs rules=eff.cap.3
fn main()
    print("io is not a capability")     # clean: printing is never denied
```

```ail error=eff.cap.1 flags=--deny,fs rules=eff.cap.1,eff.cap.2
use std.fs

fn save(text str) ! fs.IoError
    fs.write_text("out.txt", text)

fn main() !
    save("hi")?
```

```ail error=eff.cap.1 flags=--deny,all rules=eff.cap.2
use std.time

fn main()
    print(time.now_ms() > 0)
```

```ail fixture=packages/caps_denied rules=eff.cap.4,eff.cap.5
```
