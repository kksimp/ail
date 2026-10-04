# 12. Modules and programs

Chapter key: `mod`

> **Status:** Drafted (partial). Specified: modules, visibility, imports,
> single-file programs, and the entry point. Not yet designed: packages,
> manifests, and dependency resolution (build-system topic).

## Modules

**mod.unit.1** — All `.ail` files directly inside one directory form one
module. Every top-level name declared in any of its files is visible in all of
them without import. Duplicate top-level names within a module are a
compile-time error. *(ADR 0097)*

**mod.unit.2** — A module's import path is its directory path relative to the
package root, with segments separated by `.`. *(ADR 0097)*

**mod.unit.3** — A single `.ail` file given directly to `ail run` forms a
complete program by itself. *(ADR 0101)*

## Visibility

**mod.vis.1** — A top-level declaration or struct field is private to its
module unless marked `pub`. *(ADR 0098)*

**mod.vis.2** — Enum variants have their enum's visibility. Trait methods have
their trait's visibility. *(ADR 0098)*

## Imports

**mod.use.1** — `use a.b.c` makes module `a.b.c` available under the name `c`.
`use a.b.c as d` makes it available as `d`. *(ADR 0099)*

**mod.use.2** — Items of an imported module are referred to by qualified name
(`c.item`). Importing individual items, and wildcard imports, are not
supported. *(ADR 0099)*

```ail
use net.http

pub fn fetch(url str) Data ! FetchError
    resp := http.get(url)?
    decode(resp.body)?
```

## Program entry point

**mod.main.1** — A program begins executing at a function named `main` that
takes no parameters. Its return type is either absent, or an error type in the
form `!` (inferred) or `! E`. *(ADR 0088)*

**mod.main.2** — If `main` returns an error, the program writes the error
report (`err.ctx.3`) to standard error and exits with status 1. If `main`
completes normally, the exit status is 0. Traps follow `err.trap.3`.
*(ADR 0088)*

**mod.main.3** — `print(value)` is available without import. It writes the
value's text form followed by a newline to standard output. *(ADR 0088)*

```ail
fn main()
    print("Hello, world!")
```
