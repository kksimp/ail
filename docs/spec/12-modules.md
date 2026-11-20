# 12. Modules and programs

Chapter key: `mod`

> **Status:** Drafted (partial). Specified: modules, visibility, imports,
> constants, single-file programs, the package root, manifests and path
> dependencies, and the entry point. The lockfile `ail.lock` is specified in `mod.pkg.8`
> ([packages.md](../design/packages.md)). There is no registry and
> no version ranges (ADR 0383): a git `tag` is the version.

## Modules

**mod.unit.1** — All `.ail` files directly inside one directory form one
module. Every top-level name declared in any of its files is visible in all of
them without import. Duplicate top-level names within a module are a
compile-time error. *(ADR 0097)*

```ail pkg=dupes tool=check,DIR status=2 expect=mod.unit.1 rules=mod.unit.1
```

**mod.unit.2** — A module's import path is its directory path relative to the
package root, with segments separated by `.`. *(ADR 0097)*

```ail pkg=dir_path tool=run,DIR expect=9 rules=mod.unit.2
```

**mod.unit.3** — A single `.ail` file given directly to `ail run` forms a
complete program by itself: it is the root module, and its sibling files are
not part of it. It may import modules from subdirectories of its own
directory (`mod.unit.5`). *(ADR 0101, ADR 0153)*

```ail pkg=dir_path tool=run,FILE expect=9 rules=mod.unit.3
```

**mod.unit.4** — A method or associated function `fn T.name` is declared in
the module that declares `T`. Writing `fn m.T.name` is a compile-time error.
*(ADR 0155)*

```ail error=mod.unit.4
struct Point
    x int

fn m.Point.get(t Point) int
    t.x

fn main()
    print(1)
```

```ail
struct Point
    x int

fn Point.get(t Point) int
    t.x

fn main()
    print(Point.get(Point(x: 1)))
```

**mod.unit.5** — `ail run DIR` runs the package rooted at `DIR`: the `.ail`
files directly in `DIR` form the root module, which declares `main`.
`ail run FILE` runs `FILE` as the root module, with its directory as the
package root. The import path `a.b` names the directory `ROOT/a/b`. Only
imported modules are loaded. *(ADR 0153)*

```ail pkg=module_local_names tool=run,DIR expect=6 rules=mod.unit.5
```

```ail skip=context rules=mod.unit.2,mod.unit.5
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

```ail skip=context rules=mod.vis.3
pub struct Account
    pub owner str
    balance int          # private: other modules call Account.open()

pub fn Account.open(owner str) Account
    Account(owner, balance: 0)
```

The error names every private field one module uses of a struct once, with
the `pub` lines to write:

```ail pkg=private_fields tool=check,DIR status=2 expect=`sku`, rules=mod.vis.1
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
error. `use std.NAME` names a standard module instead (`std.use.1`), and
the standard modules need no `use` at all (`std.use.4`).
*(ADR 0153)*

A file `store.ail` where the directory `store/` should be is named in the
error, with where to move it:

```ail pkg=module_hints tool=check,DIR status=2 expect=store/store.ail rules=mod.use.4
```

**mod.use.5** — Imports MUST NOT form a cycle; a module can't import itself.
The error names the cycle. *(ADR 0153)*

**mod.use.6** — An import name MUST differ from the file's other import names
and from the module's top-level names. `use a.b as c` resolves a clash.
*(ADR 0154)*

**Note:** The standard modules (`use std.fs` and others) are in
[chapter 17](17-std.md).

```ail skip=context
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
computed at compile time (`ct.eval.*`); the compiler computes them before
`main` runs. A failure while computing one (overflow, division
by zero) is an error reported at the constant before the program runs.
*(ADR 0132)*

```ail skip=context
pub MAX_DEPTH := 64
KEYWORDS := {"if", "else", "while"}
SCRIPTS := [["a", "b"], ["c"]]
LIMIT := MAX_DEPTH * 2
EMPTY := Map[str, int]()
```

## Packages

**mod.pkg.1** — A package root (`mod.unit.5`) may hold a manifest named
`ail.pkg`. Every command that loads a program reads it, and the manifest of
each path dependency, before any source file. Only the root's own directory
is searched, not its parents. Without a manifest a package has no
dependencies and no capability limit. *(ADR 0230)*

```ail pkg=deps_path tool=run,DIR expect=12 rules=mod.pkg.1
```

**mod.pkg.2** — A manifest is lines of words separated by spaces; `#` starts
a comment and `"..."` quotes a word. Its first line is `package NAME` (or the `ail` line of `mod.pkg.10`),
optionally followed by `caps LIST` (`eff.cap.5`) and `unsafe allow` or
`unsafe driver` (`unsafe.block.3`, `unsafe.hw.1`), and appears once. Every other line is `dep NAME` followed
by key-value pairs: `path`, `git`, `rev`, `tag`, `caps` (`eff.cap.4`) and
`unsafe` (`unsafe allow` or `unsafe driver`, `unsafe.block.3`, `unsafe.hw.1`). A `NAME` is lowercase letters, digits and `_`.
An unknown line or key, a repeated key, or a key without a value is a
compile-time error at its position in the manifest. *(ADR 0230)*

```ail pkg=manifest_errors tool=check,DIR status=2 expect=mod.pkg.2 rules=mod.pkg.2
```

```
```ail skip=context rules=mod.pkg.1,mod.pkg.2,mod.pkg.3
package notes caps fs
dep geo path ../geo caps fs
dep textkit path vendor/textkit
```

**mod.pkg.3** — `dep NAME path DIR` makes the package rooted at `DIR`
(relative to the manifest's directory) a dependency: `use NAME` imports its
root module and `use NAME.a.b` its module `DIR/a/b`. A dependency has exactly
one of `path` and `git`, and `DIR` must exist. Inside each package, `use`
resolves against that package's root and its own manifest: importing a
package that the importing package doesn't declare is a compile-time error.
*(ADR 0231)*

```ail pkg=undeclared_dep tool=check,DIR status=2 expect=mod.pkg.3 rules=mod.pkg.3
```

**mod.pkg.4** — Package names are unique in a program: two packages with
one name must be the same directory, and a dependency with its own manifest
must be declared by its `package` name. A manifest can't declare a name
twice or declare `std`, and the root package can't have a module directory
with a package's name. *(ADR 0231)*

```ail pkg=manifest_errors tool=check,DIR status=2 expect=mod.pkg.4 rules=mod.pkg.4
```

**mod.pkg.5** — Package dependencies must not form a cycle; the error names
the cycle at the `dep` line that closes it. *(ADR 0231)*

```ail pkg=manifest_errors tool=check,DIR status=2 expect=mod.pkg.5 rules=mod.pkg.5
```

**mod.pkg.6** — `dep NAME git URL` names a repository and needs exactly one of
`rev COMMIT` and `tag TAG`; `rev` and `tag` are errors on a `path`
dependency. A git dependency is a package like a path one: `use NAME`
imports it, it resolves its own imports and manifest (`mod.pkg.3`), and its
`caps` grant applies to it unchanged (`eff.cap.4`). *(ADR 0231, ADR 0355)*

```ail pkg=manifest_errors tool=check,DIR status=2 expect=mod.pkg.6 rules=mod.pkg.6
```

```
```ail skip=context rules=mod.pkg.6
package notes
dep geo git "https://example.com/geo.git" tag v1.2 caps fs
```

**mod.pkg.7** — The toolchain, not the program, fetches a git dependency:
it runs the system `git` to clone `URL`, resolves the pin to a commit, and
stores that commit's files, without `.git`, in `$AIL_HOME/deps/NAME-COMMIT`
(`~/.ail/deps` without `AIL_HOME`). The files are read-only and a cached
commit is never fetched or changed again. Under `--deny net` (and so
`--deterministic`) nothing is fetched: a dependency missing from the cache is
an error. A missing `git`, an unreachable repository, or a pin the
repository lacks is a compile-time error at the `dep` line naming the
dependency. *(ADR 0355)*

**mod.pkg.8** — `ail.lock`, next to the root package's `ail.pkg`, holds one
line `dep NAME git URL rev|tag PIN commit COMMIT` per git dependency of the
whole graph, sorted by name. While a line matches its manifest's `git`,
`rev`/`tag`, every command uses the locked commit, so a moved tag changes
nothing. The lock is stale when it is absent with git dependencies, misses a
dependency, has a line that no longer matches, or locks a dependency no
manifest declares. `ail run` and `ail test` rewrite a stale lock after every
fetch succeeds; other commands don't write it. With `--deterministic` a stale
lock is a compile-time error. Dependencies' own `ail.lock` files are
ignored, and path dependencies aren't locked. *(ADR 0233, ADR 0356)*

**mod.pkg.9** — `ail pkg update [NAME...]`, run in a package root, resolves
the named git dependencies (every one when none is named) from their
manifest lines again, ignoring the lock, fetches them, rewrites `ail.lock`
and prints `locked NAME COMMIT` for each git dependency. Naming a dependency
that no manifest of the program declares with `git` is an error.
*(ADR 0357)*

**mod.pkg.10** — A manifest may hold one line `ail MAJOR.MINOR` (`ail 0.2`),
before or after `package`, naming the language version its sources are
written for. `ail new` writes it. Without the line, and for a single file with
no manifest, the version is the compiler's own, so only the line makes a tree
declare one. Anything but two numbers separated by `.`, a missing version and
a second `ail` line are compile-time errors at the line. A tree whose version
is lower than the compiler's is accepted and prints nothing: 0.2 only adds forms, so a tree that says `ail 0.1` still compiles. *(ADR 1084)*

```ail pkg=version_ok tool=run,DIR expect=ok rules=mod.pkg.10
```

```ail pkg=version_older tool=run,DIR expect=ok rules=mod.pkg.10
```

```ail pkg=version_errors tool=check,DIR status=2 expect=mod.pkg.10 rules=mod.pkg.10
```

**mod.pkg.11** — A package whose `ail` line names a version higher than the
compiler's (compared as MAJOR, then MINOR) is a compile-time error at the
version, whichever manifest of the program holds it, naming both versions and
saying to update the toolchain. *(ADR 1084)*

```ail pkg=version_newer tool=check,DIR status=2 expect=mod.pkg.11 rules=mod.pkg.11
```

**mod.pkg.12** — `ail fix --migrate [DIR]` reads the language version of the
tree at `DIR` (the current directory by default, as `mod.pkg.10`) and brings it
up to the compiler's version, one release step at a time. The step from 0.1 to 0.2 only rewrites the
`ail` line, because 0.2 only adds forms, and prints `migrated 0.1 to 0.2`; a
tree already at the compiler's version prints `nothing to migrate from VERSION`.
Both exit 0; a version newer than the compiler's is `mod.pkg.11` with exit
status 2. `ail doctor` shows the version of the project in the current
directory. *(ADR 1084)*

```ail pkg=version_ok tool=fix,--migrate,DIR expect=0.2 rules=mod.pkg.12
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

```ail rules=mod.main.1,mod.main.3,mod.unit.3
fn main()
    print("Hello, world!")
```

## Runnable examples for the rules above

The checks below are single files, so each one runs or fails on its own. Rules
that need a directory tree, a manifest or a git cache are cited by `skip=context`
blocks that show the layout; their checks are the package cases in
`tests/cases/packages/` (each folder's `expected.rules` names the rule
IDs it reports) and the `git_deps` gate suite.

```ail error=mod.unit.1
fn f() int
    1

fn f() int
    2

fn main()
    print(f())
```

```ail rules=mod.const.1,mod.const.2
pub MAX_DEPTH := 64
KEYWORDS := {"if", "else", "while"}
LIMIT := MAX_DEPTH * 2
EMPTY := Map[str, int]()

fn main()
    print("{MAX_DEPTH} {LIMIT} {KEYWORDS.len()} {EMPTY.len()}")
```

```ail norun rules=mod.const.3
# norun: the overflow is reported at the constant before `main` runs, so the
# block checks clean and is not run.
BIG := 9223372036854775807 + 1

fn main()
    print(BIG)
```

```ail error=mod.const.1
mut X := 1

fn main()
    print(X)
```

```ail error=mod.use.3
fn main()
    print(1)

use std.csv
```

```ail rules=mod.use.1,mod.use.2 norun
use std.csv
fn main() !
    print(csv.parse("a,b\n1,2\n")?.len())
```

```ail error=mod.use.6
use std.csv as m
use std.fs as m

fn main()
    print(1)
```

```ail error=mod.use.4
use geometry.shapes

fn main()
    print(1)
```

```ail error=mod.unit.4
struct Point
    x int

fn m.Point.area() int
    1

fn main()
    print(1)
```

```ail error=mod.vis.2
pub enum Color
    pub Red

fn main()
    print(1)
```

```ail rules=mod.vis.1
struct Account
    pub owner str
    balance int

fn main()
    a := Account("x", 0)
    print(a.owner)
```

```ail norun rules=mod.main.2
error enum Bad
    Oops "oops"

fn main() ! Bad
    return Oops
```

```ail skip=context rules=mod.use.5
# Import cycle: geometry/a.ail does `use geometry.b` and geometry/b.ail does
# `use geometry.a`. The error names the cycle (cases/packages/cycle).
```

```ail skip=context rules=mod.pkg.4,mod.pkg.5,mod.pkg.7,mod.pkg.8,mod.pkg.9
# Package graph rules: duplicate package names (cases/packages/dupes), dependency
# cycles (cases/packages/cycle), the git cache and lock (tests/git_deps.rs), and
# `ail pkg update` (tests/git_deps.rs). Shown as prose, not checked here.
```
