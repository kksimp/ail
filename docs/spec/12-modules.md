# 12. Modules and programs

Chapter key: `mod`

> **Status:** Drafted (partial). Specified: modules, visibility, imports,
> constants, single-file programs, the package root, and the entry point.
> Not yet designed: manifests and dependency resolution (build-system topic).

## Modules

**mod.unit.1** — All `.ail` files directly inside one directory form one
module. Every top-level name declared in any of its files is visible in all of
them without import. Duplicate top-level names within a module are a
compile-time error. *(ADR 0097)*

**mod.unit.2** — A module's import path is its directory path relative to the
package root, with segments separated by `.`. *(ADR 0097)*

**mod.unit.3** — A single `.ail` file given directly to `ail run` forms a
complete program by itself: it is the root module, and its sibling files are
not part of it. It may import modules from subdirectories of its own
directory (`mod.unit.5`). *(ADR 0101, ADR 0153)*

**mod.unit.4** — A method or associated function `fn T.name` is declared in
the module that declares `T`. Writing `fn m.T.name` is a compile-time error.
*(ADR 0155)*

**mod.unit.5** — `ail run DIR` runs the package rooted at `DIR`: the `.ail`
files directly in `DIR` form the root module, which declares `main`.
`ail run FILE` runs `FILE` as the root module, with its directory as the
package root. The import path `a.b` names the directory `ROOT/a/b`. Only
imported modules are loaded. *(ADR 0153)*

```
geometry/          ail run geometry
    main.ail       use geo         (the root module: main.ail and extra.ail)
    extra.ail
    geo/           module geo
        point.ail
        shapes.ail
```

## Visibility

**mod.vis.1** — A top-level declaration or struct field is private to its
module unless marked `pub`. *(ADR 0098)*

**mod.vis.2** — Enum variants have their enum's visibility. Trait methods have
their trait's visibility, and so do the methods of an `impl` block. Writing
`pub` before a variant, a variant's field, a function in a `trait` or `impl`
block, or an `impl` is a compile-time error. *(ADR 0098, ADR 0155)*

**mod.vis.3** — A struct with a private field can be built with a struct
literal only inside its own module. Reading or assigning a private field, or
calling a private method or associated function, outside its module is a
compile-time error (`mod.vis.1`). *(ADR 0155)*

```ail fragment
pub struct Account
    pub owner str
    balance int          # private: other modules call Account.open()

pub fn Account.open(owner str) Account
    Account(owner, balance: 0)
```

## Imports

**mod.use.1** — `use a.b.c` makes module `a.b.c` available under the name `c`.
`use a.b.c as d` makes it available as `d`. *(ADR 0099)*

**mod.use.2** — Items of an imported module are referred to by qualified name
(`c.item`), in expressions, types (`c.Type`), and patterns (`c.Enum.Variant`).
Importing individual items, and wildcard imports, are not supported. A bare
import name is not a value, an unknown or unimported qualifier is a
compile-time error, and `pub use` (re-exporting) is not supported.
*(ADR 0099, ADR 0154)*

**mod.use.3** — `use` declarations come first in a file, before its other
declarations. An import applies to the file that declares it.
*(ADR 0154)*

**mod.use.4** — `use a.b` requires a directory `a/b` under the package root
that contains at least one `.ail` file; otherwise it is a compile-time
error. `use std.NAME` names a standard module instead (`std.use.1`).
*(ADR 0153)*

**mod.use.5** — Imports MUST NOT form a cycle; a module can't import itself.
The error names the cycle. *(ADR 0153)*

**mod.use.6** — An import name MUST differ from the file's other import names
and from the module's top-level names. `use a.b as c` resolves a clash.
*(ADR 0154)*

**Note:** The standard modules (`use std.fs` and others) are in
[chapter 17](17-std.md).

```ail skip=bootstrap
use net.http

pub fn fetch(url str) Data ! FetchError
    resp := http.get(url)?
    decode(resp.body)?
```

## Constants

**mod.const.1** — A top-level `NAME := expr` declares a module constant. It is
visible in every file of the module (`mod.unit.1`), shares the namespace of
the module's functions and types, and is private unless marked `pub`
(`mod.vis.1`). Importers refer to it as `m.NAME` (`mod.use.2`). Its type is
the type of `expr`. Because a constant is a module-level item, `expr.bind.5`
applies to it. A top-level `mut`, `const`, `let`, `var` or `static` is a
compile-time error whose help says to write `NAME := value`, or, for `mut`,
that there are no mutable globals. *(ADR 0132)*

**mod.const.2** — `expr` MUST be a constant expression: literals (`int`,
`float`, `str`, `char`, `bool`); other constants (a cycle is a compile-time
error that names it); list, map and set literals; `List[T]()`, `Map[K, V]()`
and `Set[T]()`; struct literals and variants with constant arguments;
`none`, `[]` and `{}` where a field or element gives the expected type;
built-in operators and interpolation; and calls, method calls, field access,
indexing and ranges of compile-time-safe code (`ct.const.1–4`). A bare
`none`, `[]` or `{}` can't be a constant, since it has no expected type.
*(ADR 0132, amended by ADR 0225)*

**mod.const.3** — Constants have no order and no side effects. They are
computed at compile time (`ct.eval.*`); the bootstrap computes them before
`main` runs. A failure while computing one (overflow, division
by zero) is an error reported at the constant before the program runs.
*(ADR 0132)*

```ail skip=bootstrap
pub MAX_DEPTH := 64
KEYWORDS := {"if", "else", "while"}
SCRIPTS := [["a", "b"], ["c"]]
LIMIT := MAX_DEPTH * 2
EMPTY := Map[str, int]()
```

## Program entry point

**mod.main.1** — A program begins executing at a function named `main` that
takes no parameters. Its return type is either absent, or a result with no
success value, in the form `!` (inferred) or `! E` (`err.type.7`). ADR 0139
generalises these forms to every function, so `main` has no special return
types of its own. *(ADR 0088, ADR 0139)*

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
