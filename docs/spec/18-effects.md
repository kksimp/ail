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

## Discarded values

**eff.use.1** — A call or method call whose value is thrown away (an
expression statement that is not its block's value) MUST do something
else: a call whose value is neither unit, `Never` nor `T ! E` (`err.use.1`)
is an error when its effects (`eff.infer.1–5`) include none of `io`,
`unsafe`, `hw`, `spawn`, `mutates` or a capability, it changes no place
passed `mut` or used as a `mut` receiver, and it moves no value (a `move`
argument or a `move self` receiver). A call through a function value,
a method of a channel, task, mutex or atomic, a function without a body,
a call inside an `unsafe` block, and the last statement of a block in a
function that can fail (its error value, `err.ret.*`) are never reported. `_ := expr` throws a
value away on purpose. The fix offers the in-place twin of a copying
built-in (`xs.sorted()` → `xs.sort()`) when there is one, and `_ := ` always.
*(ADR 1082)*


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

**eff.cap.6** — `@caps(LIST)` on a function limits it to the listed
capabilities, and so everything it calls: the function MUST NOT have, by the
inferred effects of `eff.infer.*`, a capability outside `LIST`. `@caps(none)`
allows none. Only capabilities count; `io`, `fail`, `mutates`, `unsafe` and
`spawn` are not limited (`@pure` limits those). The diagnostic is at the
attribute and names, for each capability outside the list, the shortest chain
of calls that adds it, such as `report -> save -> fs.write_text`. A caller of
the function sees its inferred set as before. *(ADR 1089)*

```ail error=eff.cap.6 rules=eff.cap.6
use std.fs

fn save(path str, text str) ! fs.IoError
    fs.write_text(path, text)

@caps(none) fn render(path str) str ! fs.IoError
    save(path, "x")?              # error: `render` is limited to no capability
    "done"

fn main() !
    print(render("out.txt")?)
```

**eff.cap.7** — `LIST` is `none`, or a comma-separated list of the names
`--deny` takes (`eff.cap.2`) without `none`. Any other entry, an empty list,
or `none` with other names is a compile-time error at the attribute.
*(ADR 1089)*

```ail error=eff.cap.7 rules=eff.cap.7
@caps(io) fn show(x int)
    print(x)

fn main()
    show(1)
```

```ail rules=eff.cap.6
use std.fs

@caps(fs) fn save(path str, text str) ! fs.IoError
    fs.write_text(path, text)

@caps(none) fn double(x int) int
    x * 2

fn main() !
    save("out.txt", "{double(21)}")?
```

## Scoped allow-lists

`--deny` (`eff.cap.1`) is all or nothing for a capability. `--allow` is its
complement for running code you did not write: it names what the program may
reach, and how far. *(ADR 1072)*

**eff.allow.1** — `ail run --allow LIST FILE` and `ail check --allow LIST FILE`
treat `LIST` as an allow-list. At compile time every capability the list does
not name is denied as by `--deny` (`eff.cap.1`), and so is unsafe code, which
no scope can contain (`eff.allow.9`), unless the list has `unsafe`. At run time
every standard-library call that reaches the file system, the network, a
program or an environment variable is checked against the scopes
(`eff.allow.3-6`). `--allow` may be repeated, and combines with `--deny`.
`--allow none` allows nothing: the program may print and compute. Printing and
standard input and output are never restricted. Nothing changes for a
program run without `--allow`.

```ail flags=--allow,none rules=eff.allow.1
fn main()
    print("computing is always allowed")
```

```ail error=eff.cap.1 flags=--allow,fs.read=./data rules=eff.allow.1
use std.time

fn main()
    print(time.now_ms() > 0)       # error: `clock` is not in the allow-list
```

**eff.allow.2** — An entry is `NAME` (the capability with no limit) or
`NAME=SCOPE`. The names are `fs`, `fs.read`, `fs.write`, `net`, `net.serve`,
`proc`, `env`, `clock`, `random`, `ui`, `app` and `unsafe`; only the first
seven take a scope, which is the text after the first `=` and holds no comma
or space. An unknown name, a scope on a name that takes none, or an empty
scope is a usage error (exit status 2) that lists the names. `fs.read` and
`fs.write` allow `fs` at compile time, and `net.serve` allows `net`; `fs`
alone allows both reading and writing anywhere. Repeating a name adds scopes.

```ail error=eff.cap.1 flags=--allow,fs rules=eff.allow.2
use std.net

fn main() !
    print(net.get("http://example.com")?.status)   # error: `net` is not in the allow-list
```

**eff.allow.3** — `fs`, `fs.read` and `fs.write` scopes are path prefixes. A
call that reads (`read_text`, `read_bytes`, `read_range`, `map`, `exists`,
`list_dir`, `walk`, `scan`, `glob`, `metadata`, `mode`, `read_link`, `volume`,
the source of `copy`, and `compress.gzip_file`, `gunzip_file`, `zip_dir`
and `unzip` for their input) needs `fs` or `fs.read` to cover the path. A
call that writes, removes or changes (`write_*`, `append_*`, `remove`,
`make_dir`, `make_temp_dir`, both ends of `rename`, the target of `copy`,
`trash`, `set_mode`, `set_modified`, `symlink`, `truncate`, `write_at` and the
outputs of the compress functions) needs `fs` or `fs.write`. A path is
covered when, after the program's path and the scope are both made absolute
(a relative one against the working directory), have every symbolic link
resolved and `.` and `..` applied as the operating system applies them, the
path equals the scope or lies below it. So `data/../secret` and a link inside
`data` that points outside are refused. On Windows the system names the path
(links, junctions and short names resolved) and a UNC or device path, an
alternate stream (`name:stream`), a reserved device name and a drive-relative
path (`C:name`) are refused. On macOS and Windows the comparison ignores
case, as the default volumes do. A glob is read from its first component with
a wildcard, and a `..` after a wildcard is refused.

```ail flags=--allow,fs.read=./data rules=eff.allow.3
use std.fs

fn main() !
    print(fs.read_text("data/a.txt")?.len())   # allowed at run time: under ./data
```

**eff.allow.4** — A `net` scope is `HOST`, `HOST:PORT`, `*.DOMAIN` or `HOST:*`
(a bracketed `[::1]` for IPv6; a port is `*` or a number, and absent means any).
A request or `connect` to `host:port` needs a scope that covers it, matching
the host as the program wrote it, without regard to case or a trailing dot. A
name is never matched against the address it resolves to, and an address
written as a number is never matched against a name: to reach `93.184.216.34`
the scope must say so. Each redirect is a new request and is checked again.
`net.serve` takes `PORT` or `HOST:PORT` and covers `net.listen` and `net.serve`
on that address.

```ail flags=--allow,net=api.example.com:443 rules=eff.allow.4
use std.net

fn main() !
    print(net.get("https://api.example.com/v1/items")?.status)
```

**eff.allow.5** — A `proc` scope is a program name exactly as the program
gives it to `proc.run`, `run_bytes`, `start` or `which`: `git` allows `git`
and not `/usr/bin/git` or `./git`. On Windows case and a trailing `.exe` are
ignored. A program the policy allows runs with that program's own power, not
the policy's (`eff.allow.9`).

```ail flags=--allow,proc=git rules=eff.allow.5
use std.proc

fn main() !
    print(proc.run("git", ["--version"])?.out)
```

**eff.allow.6** — An `env` scope is a variable name, or a prefix when it ends
in `*` (`env=AIL_*`). `env.get(name)` and `env.home()` (which reads `HOME`, and
`USERPROFILE` on Windows) need a scope that covers the variable; `env.vars()`
returns only the variables a scope covers. Windows ignores case. `env.args`,
`env.cwd` and `env.program` are gated at compile time only.

```ail flags=--allow,env=HOME rules=eff.allow.6
use std.env

fn main()
    print(env.get("HOME") else "")
```

**eff.allow.7** — A call the policy does not allow fails closed: the program
ends at once, after flushing its output, with exit status 1 and one line on
standard error, `trap[eff.allow.N]: CAPABILITY of ARGUMENT by FUNCTION is not
allowed; allow it with --allow ENTRY (this run allows: LIST)`, where `N` is the
rule of the capability, `ARGUMENT` is the canonical path or the host and port,
and `ENTRY` is the one entry that would allow exactly that use. It is not an
error value the program can catch or ignore. An entry the run-time reader does
not understand allows nothing.

```ail flags=--allow,fs.write=./out rules=eff.allow.7
use std.fs

fn main() !
    fs.write_text("out/log.txt", "ok")?   # a write outside ./out would end with trap[eff.allow.3]
```

**eff.allow.8** — The run-time half reads the environment variable
`AIL_ALLOW`, which holds the same list. `ail run --allow` sets it for the
program only, after the program is built and checked, so the compiler is not
restricted; an executable started by hand with `AIL_ALLOW` set obeys it the
same way. A program started with no `AIL_ALLOW`, or an empty one, is not
checked. A child process that a program starts inherits the variable, which
restricts a child that is itself an AIL program.

```ail flags=--allow,fs.read=.,--deny,net rules=eff.allow.8
use std.fs

fn main()
    print(fs.exists("main.ail"))
```

**eff.allow.9** — `--allow` is a check on the standard library's calls, not a
sandbox. It does not protect against: unsafe code, C calls and `std.mem` or
`std.hw` access (refused at compile time unless `unsafe` is listed, and
unrestricted when it is); a program that `proc` allows, which runs with its own
authority (`git -c`, an interpreter, a shell); a race on the file system
between the check and the use, by another process or task that can change a
link; hard links, bind mounts and other names for the same file that the
operating system makes; a case-sensitive volume on macOS or Windows, where
names that differ only in case are one scope; the name a host resolves to
(DNS rebinding) and a proxy named by `HTTP_PROXY`, which is connected to in
place of the host; names that glob lists through a link inside a scope; the
working directory and program path, which `env` shows; resource use (time,
memory, disk and network volume; see `--timeout` and `--max-memory`); and the
capabilities `ui`, `app`, `clock` and `random`, which are allowed or denied but
not scoped. For untrusted code add an operating-system sandbox as well.

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

```ail error=eff.use.1
fn area(w int, h int) int
    w * h

fn main()
    area(3, 4)                  # error: the value is thrown away, and area does nothing else
    print("done")
```

```ail rules=eff.use.1
fn noisy(n int) int
    print("noisy {n}")
    n

fn main()
    mut xs := [3, 1, 2]
    noisy(1)                    # clean: noisy prints
    xs.pop()                    # clean: pop changes xs
    _ := xs.len()               # clean: thrown away on purpose
    print(xs)
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
