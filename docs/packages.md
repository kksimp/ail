# Packages

How to split a program into packages, use someone else's code, and say what
that code may touch. This is the user guide; the design and its rejected
alternatives are in [design/packages.md](design/packages.md), the rules in
[spec/12-modules.md](spec/12-modules.md#packages) (`mod.pkg.*`) and
[spec/18-effects.md](spec/18-effects.md#denying-capabilities) (`eff.cap.4-5`).

There is **no registry** ([ADR 0383](decisions/0383-no-package-registry.md)): a
dependency is a directory or a git repository, named by one line in your
manifest. Nothing is hosted, and there is no version solver.

## A package is a directory

`ail run DIR` makes `DIR` a package. Its `.ail` files are its root module and
its subdirectories are modules (`use a.b`). A single file needs no manifest;
`ail new NAME` makes a directory with an `ail.pkg`, a first program, and an agent kit (`AGENTS.md`, `.mcp.json`; `--no-agent` skips the kit).

## The manifest: `ail.pkg`

`ail.pkg` sits in the package root. It is read from that directory only, never
from a parent. One word per field, one line per fact, `#` starts a comment:

```text
package notes caps fs
dep geo path ../geo caps fs
dep textkit path vendor/textkit
dep json git https://github.com/example/ail-json tag v1.4 caps fs,env
```

| Line | Meaning |
|---|---|
| `package NAME [caps LIST]` | First line, once. `NAME` is lowercase letters, digits and `_`. `caps` limits what the package's own code may reach (see below). |
| `dep NAME path DIR [caps LIST]` | A dependency in `DIR`, relative to the manifest. |
| `dep NAME git URL rev COMMIT [caps LIST]` | A git dependency pinned to a commit. |
| `dep NAME git URL tag TAG [caps LIST]` | The same, pinned to a tag. |

An unknown line or key, a repeated key, or a missing value is a compile-time
error at that line (`mod.pkg.2`). A git dependency needs exactly one of `rev`
and `tag` (`mod.pkg.6`).

## Using a dependency

`dep geo ...` lets the package write `use geo` for the dependency's root
module and `use geo.shapes` for its module `shapes`. Names are then qualified:
`geo.describe(p)`, `shapes.area(r)`.

Each package sees only the packages **it** declares (`mod.pkg.3`). If `app`
uses `tally` and `tally` uses `textkit`, `app` can import `textkit` only by
declaring it too. A package name means one directory in the whole program
(`mod.pkg.4`), and dependencies can't form a cycle (`mod.pkg.5`).

## A worked example: two packages

This is the tree `tests/cases/packages/doc_two_packages/`, run by the gate, so
it stays true. `wordtool` depends on `tally`, and `tally` depends on `textkit`:

```text
wordtool/
  ail.pkg
  main.ail
  vendor/
    tally/
      ail.pkg
      tally.ail
    textkit/
      textkit.ail
```

`wordtool/ail.pkg` declares both packages the program imports, and grants
nothing (`caps none`):

```text
package wordtool caps none
dep tally path vendor/tally
dep textkit path vendor/textkit
```

`wordtool/main.ail`:

```ail skip=context
use tally
use textkit

fn main()
    r := tally.summarize("The quick brown fox jumps")
    print("{r.total} words, longest {r.longest}")
    print(textkit.shout(r.longest))
```

`vendor/tally/ail.pkg` declares its own dependency, by the same name and
directory the app uses (`mod.pkg.4`):

```text
package tally
dep textkit path ../textkit
```

`vendor/tally/tally.ail`:

```ail skip=context
use textkit

pub struct Summary
    pub total int
    pub longest str

pub fn summarize(text str) Summary
    ws := textkit.words(text)
    mut best := ""
    for w in ws
        if w.len() > best.len()
            best = w
    Summary(total: ws.len(), longest: best)
```

`vendor/textkit/textkit.ail` has no manifest, so it has no dependencies and no
limit of its own:

```ail skip=context
pub fn words(s str) List[str]
    s.to_lower().words()

pub fn shout(s str) str
    "{s.to_upper()}!"
```

Run it from the directory that holds `wordtool/`:

```sh
./ail run wordtool
```

```text
5 words, longest quick
QUICK!
```

## Capabilities: what a dependency may reach

A dependency can't reach the filesystem, environment, processes, clock,
randomness or network unless its `dep` line says so, and this is checked at
compile time, before anything runs:

- The names are `fs`, `env`, `proc`, `clock`, `random`, `net`, `net.serve`,
  `ui` and `app`. `all` and `none` are also accepted (`eff.cap.2`).
- A `dep` line without `caps` grants **nothing**. Printing is not a
  capability (`eff.cap.3`).
- A grant is an upper bound for the dependency and everything below it: if
  `tally` used a package that reads the environment, you would grant `env` to
  `tally`.
- `caps` on the `package` line limits the package's own code the same way. On
  the root package it is an allow-list for the whole program.
- `ail new` writes `caps fs=.,fs=@tmp,env,clock,random` on the `package` line: `ail run`
  lets the program reach files under the project and the system temp directories (`fs=@tmp`) and no network or other
  program until the list gains the entry. A path scope is relative to the
  manifest. Delete the `caps` list to lift the limit.
- Every function of the package counts, not only those `main` reaches.

A violation is an error at the manifest line, with the chain of calls:

```text
ail.pkg:3:5: error[eff.cap.4]: dependency `geo` reaches the capability `fs`,
which this `dep` line doesn't grant: geo.size -> fs.exists
help: add `caps fs` to this `dep` line if the package should reach `fs`;
otherwise remove the call
```

Either add the capability to that line, or remove the call. `ail effects FILE`
lists what a program reaches. This is compile-time denial, not a sandbox:
`unsafe` code needs its own grant (`unsafe allow` on the line, `unsafe.block.3`).

## Git dependencies and `ail.lock`

```text
package app
dep json git https://github.com/example/ail-json tag v1.4
```

- The **toolchain** fetches the repository with the system `git`, so `git`
  must be on `PATH`. The files (without `.git`) go read-only into
  `$AIL_HOME/deps/NAME-COMMIT`, `~/.ail/deps` by default (`%USERPROFILE%\.ail\deps`
  on Windows). Projects share the copy, and a cached commit is never fetched
  again (`mod.pkg.7`).
- `ail run` and `ail test` write `ail.lock` next to the root `ail.pkg` when it
  is missing or stale. It holds one line per git dependency of the whole graph:

  ```text
  dep json git https://github.com/example/ail-json tag v1.4 commit 4f2a9c1e5b...
  ```

  **Commit it.** While a line matches the manifest, every command builds the
  locked commit, so a moved tag changes nothing (`mod.pkg.8`).
- `ail pkg update [NAME...]` resolves the named dependencies (all if none)
  from the manifest again, rewrites the lock and prints `locked NAME COMMIT`
  (`mod.pkg.9`).
- `--deterministic` turns a stale lock into an error, and `--deny net` fetches
  nothing: a dependency missing from the cache is an error. Use them in CI.
- Path dependencies are part of your tree and are not locked. A dependency's
  own `ail.lock` is ignored.

## Not provided

- A registry, version ranges or a solver (`dep json 1.4`): pin a tag or commit.
- Two versions of one package in one program.
- Build scripts and features: a build script would run code that a capability
  grant can't see.

See [design/packages.md](design/packages.md) for the reasons and the open
items.

## Language version

`ail 0.2` in `ail.pkg` names the language version a tree is written for (`mod.pkg.10`); `ail new`
writes it. A tree that needs a newer language than the compiler is an error (`mod.pkg.11`).
A tree that says `ail 0.1` is still accepted; the few changes that can break a 0.1 program are in the changelog. `ail fix --migrate [DIR]`
rewrites a 0.1 tree's `ail` line to `ail 0.2` and prints `migrated 0.1 to 0.2`; at the current version it
prints `nothing to migrate from 0.2` (`mod.pkg.12`, [ADR 1171](decisions/1171-language-version-0-2.md)).
