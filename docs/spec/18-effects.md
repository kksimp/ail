# 18. Effects and capabilities

Chapter key: `eff`

> **Status:** Drafted (partial). Specified: inferred effect sets, `@pure`, and
> denying capabilities to a program. Not yet designed: effects written in
> signatures and capability manifests for packages. Deterministic mode is
> specified in the last section.

## Inference

**eff.infer.1** — Every function has an *effect set*: the effects `io`,
`fail`, `mutates`, `unsafe` and `spawn`, and the *capabilities* `fs`, `env`,
`proc`, `clock`, `random` and `net`. The set is inferred; it is never written in
source. *(ADR 0163)*

**eff.infer.2** — A function has `fail` if its return type is `T ! E`, `! E`,
`!T` or `!`, and `mutates` if it has a `mut` parameter or `mut self`. Both
come from its own signature only. *(ADR 0163)*

**eff.infer.3** — A function has every effect other than `fail` and
`mutates` that any function it may call has. The effect sets of a program
are the least sets that satisfy this rule (a fixed point over the call
graph, so recursion is allowed). Constants contribute nothing, because a
constant expression runs no code (`mod.const.2–3`). *(ADR 0163)*

**eff.infer.4** — The sources of effects are: `print` gives `io`; the
standard modules give their module's effects: `fs.*` gives `fs` and `io`,
`env.*` gives `env`, `io.*` gives `io`, `proc.*` gives `proc` and `io`, and
`time.*` gives `clock`, `random.*` gives `random`; an `unsafe` block gives `unsafe`; `spawn` gives
`spawn`. Nothing gives `net` yet. *(ADR 0163)*

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

**eff.pure.1** — A function marked `@pure` MUST NOT have `io`, `unsafe`,
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
lists the capabilities. *(ADR 0165)*

**eff.cap.3** — `io` and the other effects are not capabilities and can't be
denied: printing is how a program reports, and failure and mutation are
already in signatures. *(ADR 0165)*

## Deterministic execution

**eff.det.1** — A program whose `main` reaches none of the capabilities
`clock`, `random`, `proc` and `net`, and has no `spawn`, produces the same
output on every run given the same inputs: its arguments, environment
variables, files and standard input. Map and set iteration is in insertion
order (`type.map.1`), and hash values are never visible, so neither can vary
a run. Programs with tasks are outside this rule until task scheduling is
specified. *(ADR 0207)*

**eff.det.2** — `ail run --deterministic FILE` and `ail check
--deterministic FILE` are `--deny clock,net,proc,random` (`eff.cap.1`), and
combine with `--deny`. `fs`, `env` and standard input are inputs, not
sources of nondeterminism, so they stay allowed. *(ADR 0207)*

**eff.det.3** — `ail run --seed S FILE` seeds `std.random` (`std.random.3`).
A program that uses `random` and runs with a seed repeats without being
denied the capability. *(ADR 0207)*
