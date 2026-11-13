# compiler: pkg

## pkg
```
pub fn pkg.command(args List[str]) int [clock env fs io proc]  ## Run `ail pkg ARGS`; the exit status. Only `update` exists.
pub pkg.LOCK := "ail.lock"  ## The lockfile's name (mod.pkg.8).
pub struct pkg.Options  ## How the command that loads the program treats fetching and the lock. The
    pub write_lock bool  ## Write `ail.lock` when it is absent or stale (`run`, `test`).
    pub frozen bool  ## A stale lock is an error (mod.pkg.8).
    pub offline bool  ## Use only what the cache holds (mod.pkg.7).
    pub refresh List[str]?  ## These dependencies (all if empty) ignore the lock (mod.pkg.9).
pub fn pkg.options() pkg.Options [env]
pub struct pkg.Entry  ## One `ail.lock` line: `dep NAME git URL rev|tag PIN commit SHA`.
    pub name str
    pub source pkg.Source
    pub commit str
pub struct pkg.Lock  ## The lock as read, and the entries this load used.
    path str
    present bool
    text str
    entries List[pkg.Entry]
    pub used List[pkg.Entry]
    pub failed bool  ## A fetch failed, so the lock isn't rewritten.
pub fn pkg.read_lock(root str) pkg.Lock [fs io]  ## Read `ail.lock` in `root`. A malformed line is ignored: the lock is
pub pkg.LANG_MAJOR := 0  ## The language version this compiler implements: `ail MAJOR.MINOR` in a manifest names one (mod.pkg.10).
pub pkg.LANG_MINOR := 1
pub fn pkg.lang_version() str  ## `0.1`, the version text of this compiler's language.
pub pkg.CAPABILITIES := ...  ## The capabilities `caps` can grant (eff.cap.1).
pub struct pkg.Source  ## A `git` dependency's source: `git URL` and its `rev` or `tag` pin (mod.pkg.6).
    pub url str
    pub kind str  ## `rev` or `tag`
    pub pin str
pub struct pkg.Dep
    pub name str
    pub pos ast.Pos
    pub dir str?  ## `path DIR`; none for a git dependency or an error.
    pub git pkg.Source?
    pub caps List[str]
    pub unsafe_ok bool  ## `unsafe allow` on the line (unsafe.block.3).
    pub driver_ok bool  ## `unsafe driver` on the line (unsafe.hw.1).
pub struct pkg.Manifest
    pub name str?  ## The `package` line's name and where it is; none for an empty file.
    pub name_pos ast.Pos
    pub caps List[str]?
    pub unsafe_ok bool
    pub driver_ok bool
    pub deps List[pkg.Dep]
    pub version str?  ## The `ail MAJOR.MINOR` line's version and where it is; none without the line (mod.pkg.10).
    pub version_pos ast.Pos
pub struct pkg.Parsed
    pub manifest pkg.Manifest
    pub diags List[syntax.Diag]
pub fn pkg.newer_than_compiler(parts (int, int)) bool  ## Is version `parts` newer than this compiler's language (mod.pkg.11)?
pub fn pkg.parse(src str) pkg.Parsed  ## Parse a manifest (mod.pkg.2-3, mod.pkg.6). Positions are local.
pub fn pkg.tree_version(dir str) str [fs io]  ## The language version a tree declares: its `ail.pkg`'s `ail` line, or this compiler's own when there
pub fn pkg.migrate_command(dir str) int [fs io]  ## The exit status of `ail fix --migrate [DIR]`: 2 for a version newer than this compiler, else 0 with
pub pkg.MANIFEST := "ail.pkg"  ## The manifest's file name (mod.pkg.1).
pub struct pkg.Packages  ## Every package of a program: the root package (named `""`) and its
    roots Map[str, str]  ## Package name to root directory.
    deps Map[str, List[str]]  ## Package name to the dependencies its manifest declares.
    manifests List[pkg.ManifestFile]
    lock pkg.Lock
    opts pkg.Options
    pub fn pkg.Packages.fetch_failed(self) bool  ## A git dependency's fetch failed (mod.pkg.7), so its modules are missing for that reason alone.
    pub fn pkg.Packages.locked(self) List[pkg.Entry]  ## The git dependencies resolved, for `ail pkg update` (mod.pkg.9).
    pub fn pkg.Packages.target(self, from str, parts List[str]) pkg.Target  ## The module that the import path `parts`, written in module `from`, names
    pub fn pkg.Packages.finish(self, mut prog ast.Program, mut diags List[syntax.Diag], mut texts List[str]) [mutates]  ## Add the manifests to the program's files, after its source files, with
pub struct pkg.Target  ## What an import path written in some module names.
    pub module str  ## The module's qualified name in the program (`geo.shapes`).
    pub dir str
    pub undeclared bool  ## The import names a package that the importing package doesn't declare (mod.pkg.4).
pub fn pkg.read(root str) pkg.Packages [clock env fs io proc]  ## Read the manifest at `root`, if any, and every path dependency's.
pub fn pkg.read_with(root str, opts pkg.Options) pkg.Packages [clock env fs io proc]  ## `read` with explicit fetch and lock options (`ail pkg update`).
pub fn pkg.update(packages pkg.Packages, names List[str]) (List[str], List[syntax.Diag])  ## The lines `locked NAME COMMIT` of the dependencies resolved, or the
```

trimmed to fit 2000 tokens (~1575); dropped: private items
