# Packages, dependencies and capability manifests

Status: design for Topic 13 (package management), 2026-10-20. Decided in
[ADRs 0230–0233](../decisions/README.md); normative rules in
[12-modules.md](../spec/12-modules.md#packages) (`mod.pkg.*`) and
[18-effects.md](../spec/18-effects.md#denying-capabilities) (`eff.cap.4–5`).
The bootstrap implements path and git dependencies, capability grants and
the lockfile (ADRs 0355–0357); a registry is designed here only.

## Goals

1. An agent adding a dependency writes **one line** and can read every
   dependency, and what it may reach, in one small file (wishlist 1.3, 5.1).
2. A dependency can't reach the filesystem, environment, processes, clock,
   randomness or network unless that line grants it, checked **before
   anything runs** by the same inference as `--deny` (wishlist 5.1, 6.6;
   resolves OQ-045).
3. Builds are reproducible: the same manifest, lockfile and sources give
   the same program and the same cache keys (ADR 0015).
4. Nothing changes for single-file programs and manifest-less packages
   (ADRs 0101, 0153).

## The manifest: `ail.pkg`

A package root may hold `ail.pkg`. It is a line format: words separated by
spaces, `#` comments, `"..."` for a word with spaces.

```
# ail.pkg
package notes caps fs
dep geo path ../geo caps fs
dep textkit path vendor/textkit
dep json git https://github.com/example/ail-json rev 4f2a9c1e caps fs,env
```

- `package NAME [caps LIST]` comes first, once. `NAME` is a module-segment
  name (lowercase, digits, `_`). `caps` on this line limits the package's
  own code, including what it reaches through its dependencies (an
  allow-list for the whole program when it is the root). Without it, the
  root package is unlimited, as today.
- `dep NAME SOURCE [caps LIST]`, one per dependency. `SOURCE` is
  `path DIR` (relative to the manifest) or `git URL rev COMMIT` /
  `git URL tag TAG`. Without `caps`, a dependency gets **no** capabilities.
- `LIST` is `fs,env,...`, `all` or `none` (the names of `--deny`).

Keys are fixed per line kind; an unknown line, key, or capability is an
error, all reported in one pass with the file and line.

### Format measurement

The same 4-dependency manifest (2 path, 2 git, 3 with capabilities) in each
candidate format, `tools/tokbench/run.sh tools/tokbench/corpus/r9/*`, mean of
five tokenizers:

| Format | File | Mean tokens | vs lines |
|---|---|---:|---:|
| One line per dependency (chosen) | `a-lines.pkg` | 72.0 | — |
| Indented blocks, one key per line | `b-indented.pkg` | 89.0 | +24% |
| Dotted key = value lines | `g-lines-kv.pkg` | 107.4 | +49% |
| TOML inline tables | `c-toml.toml` | 113.6 | +58% |
| TOML one table per dependency | `d-toml-tables.toml` | 118.0 | +64% |
| JSON | `e-json.json` | 128.6 | +79% |
| AIL struct literal | `f-ail-literal.ail` | 131.4 | +83% |

The line format also wins on AI understanding: one dependency is one line, so
`grep '^dep '` lists them, a diff adding a dependency is one line, and a
diagnostic points at the exact line to change. The AIL-literal form was the
brief's other candidate; it costs the most (quotes, `name:` labels, commas)
and would need the parser to run before the loader knows where packages are.

## Packages and roots

- `ail run DIR` makes `DIR` the package root; `ail run FILE` makes the file's
  directory the root (ADR 0153). `ail.pkg` is read from that root only, never
  from a parent directory, so `ail run examples/x.ail` is unaffected by
  anything above `examples/`.
- A single-file program (ADR 0101) needs no manifest. A directory of `.ail`
  files without `ail.pkg` is a package with no dependencies and no limit.
- A dependency's directory is a package root of its own: its `.ail` files are
  its root module, and its subdirectories its modules. It may have its own
  `ail.pkg`; if it does, its `package` name must equal the `dep` name.

## Naming and imports

`dep geo ...` makes `use geo` import the dependency's root module and
`use geo.shapes` its module `shapes`. Inside a package, every `use` resolves
against **that package's** root and **its own** manifest: a package sees only
the packages it declares (`mod.pkg.3`), so a transitive dependency must be
declared to be imported.

Package names are global to a program: two packages with one name must be the
same directory, and the root package can't have a module directory with a
package's name (`mod.pkg.4`). This keeps one name per package everywhere an
agent reads it (diagnostics, text forms such as `geo.Point(...)`, effect
chains), and it is what the bootstrap's qualified-name scheme (ADR 0156)
needs. `use geo as g` still renames within one file. Package dependencies
can't form a cycle (`mod.pkg.5`), for the same layering reasons as module
imports (ADR 0153).

## Capabilities per dependency

Every function belongs to one package: the dependency whose name is the first
segment of its module, otherwise the root package. A grant is the `caps` of a
`dep` line (for that dependency) or of a `package` line (for that package).
After type checking, the effect inference that `--deny` uses (ADRs
0163–0165) gives each function its capabilities; a function of package P
that reaches a capability that one of P's grants doesn't include is an error
**at the manifest line**:

```
ail.pkg:3:5: error[eff.cap.4]: dependency `geo` reaches the capability `fs`,
which this `dep` line doesn't grant: geo.size -> fs.exists
help: add `caps fs` to this `dep` line if the package should reach `fs`;
otherwise remove the call
note: ...:4:8: `geo.size` reaches `fs` (in vendor/geo/geo.ail)
```

- A package's capabilities include those of its dependencies: if `geo`
  depends on `clock` with `caps env`, then `geo` reaches `env`, and the app
  must grant `env` to `geo` too. A grant is therefore an upper bound for a
  whole subtree, readable from one line.
- Every function of the package counts, not only those `main` reaches: the
  grant constrains the package, not one program's use of it. Only loaded
  (imported) modules are checked.
- When a package is declared twice (a diamond), each grant is checked, so the
  stricter one wins.
- The inference is conservative (`any Trait` counts every implementation,
  `eff.infer.5`), so a dependency calling back into a trait the app
  implements with `fs` is charged `fs`. That is the safe direction; the fix
  is to grant it or restructure.

This is compile-time denial, not a sandbox: it covers what the language can
reach. `unsafe` and FFI (Topic 14–15) bypass inference; when they land, a
dependency using either needs an explicit `caps unsafe` grant (open item
below).

## Versions, git dependencies and the lockfile

- A git dependency pins exactly one `rev` (commit) or `tag`. There are no
  version ranges and no solver: one exact pin per package, and a diamond with
  two different pins for one name is an error naming both lines.
- `ail.lock`, next to `ail.pkg`, records every git dependency of the whole
  graph in the same line format, written by `ail run` and `ail test` when
  stale (ADR 0356; the design's `sum` was dropped, the commit names the tree):

  ```
  dep json git https://github.com/example/ail-json tag v1.4 commit 4f2a9c1e5b...
  ```

  `commit` is the full hash the pin resolved to. A build uses the locked
  commit while the line matches the manifest, and never reaches the network
  once the cache holds the tree. `--deterministic` rejects a stale lock;
  `ail pkg update [NAME]` re-pins (ADR 0357). Path dependencies are part of
  the working tree and aren't locked.
- The system `git` fetches each tree into `$AIL_HOME/deps/NAME-COMMIT`
  (`~/.ail/deps` by default), without `.git` and read-only, so projects share
  one copy and a tree is never edited in place (ADR 0355).

## Reproducible builds and cache keys (ADR 0015)

The whole-program incremental cache keys each package's analysis by: the
compiler version, the package's `sum` (git) or source content hash (path),
its grants, and the keys of its dependencies. A grant is part of the key
because it changes which programs are accepted. With a lockfile, the same
inputs therefore give the same keys on every machine; keys hold no
timestamps or absolute paths.

## What is deferred, and why

- **A registry** (`dep json 1.4` without a URL) is a **serious decision** for
  the designer: it involves hosting, accounts, naming policy and publishing
  (AGENTS.md escalation criterion 4), plus version ranges and a solver. The
  line format leaves room for it: a `version` key on a `dep` line.
- **Features, build scripts, per-target dependencies**: not planned. Build
  scripts would run arbitrary code at build time, which a capability grant
  can't see.

## Rejected alternatives

- **TOML or JSON manifests:** 58–79% more tokens on the sample, and TOML's
  table forms give an agent two ways to write the same thing.
- **An AIL-literal manifest:** the most tokens (83% more), and the loader
  would need the parser before it knows where packages are.
- **Capabilities as values passed from `main` (OQ-045 option b, wishlist
  6.6):** the most explicit, but every call chain to a dependency carries the
  capability parameter, a token cost on every call for a check the inference
  already makes.
- **No package-level control (OQ-045 option c):** an AI-added dependency could
  reach anything `main` can.
- **Runtime denial per dependency:** partial effects before the trap; the same
  reasons as ADR 0165.
- **Default grant `all` for dependencies:** convenient, but the safe default is
  the point of wishlist 5.1; granting is one word.
- **Per-package namespaces (two versions of one package in one program):**
  needs a mangling scheme the bootstrap doesn't have, and makes a name in a
  diagnostic ambiguous. Revisit with the registry.

## Open items

- Verify a cached tree against its commit (the read-only files guard against
  accidental edits only).

- `caps unsafe` / `caps ffi` once unsafe code and FFI are implemented.
- Whether `ail effects` should print each package's reached capabilities
  next to its grant (a one-command audit).
- `ail api DIR` across packages (lane 3's known gap).
