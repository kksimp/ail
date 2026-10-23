# compiler: pkg

## pkg
```
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
pub struct pkg.Parsed
    pub manifest pkg.Manifest
    pub diags List[syntax.Diag]
struct pkg.Word
    text str
    pos ast.Pos
fn pkg.diag(rule str, at ast.Pos, msg str, help str) syntax.Diag
fn pkg.is_space(c char) bool
fn pkg.words(line str, n int, mut diags List[syntax.Diag]) List[pkg.Word] [mutates]  ## The words of one line, with their columns: whitespace separates them,
fn pkg.is_name(s str) bool
fn pkg.is_lower(c char) bool
fn pkg.is_digit(c char) bool
fn pkg.caps(list str, at ast.Pos, mut diags List[syntax.Diag]) List[str] [mutates]  ## `fs,env`, `all` or `none` (eff.cap.2): the capabilities, sorted.
struct pkg.Pair  ## The value of key `key` among a line's pairs, with the key's position.
    key str
    value str
    pos ast.Pos
fn pkg.find_pair(pairs List[pkg.Pair], key str) pkg.Pair?
pub fn pkg.parse(src str) pkg.Parsed  ## Parse a manifest (mod.pkg.2-3, mod.pkg.6). Positions are local.
pub pkg.MANIFEST := "ail.pkg"  ## The manifest's file name (mod.pkg.1).
pkg.FILE_LINES := 1048576  ## Each file's lines are numbered from a multiple of this (modules.rs).
struct pkg.ManifestFile
    package str
    path str
    src str
    diags List[syntax.Diag]  ## Positions are local to the manifest until `finish`.
    grants List[ast.Grant]
    unsafe_ok List[str]  ## The packages this manifest lets contain `unsafe` code (unsafe.block.3).
    driver_ok List[str]  ## The packages this manifest lets reach hardware: `unsafe driver` (unsafe.hw.1).
pub struct pkg.Packages  ## Every package of a program: the root package (named `""`) and its
    roots Map[str, str]  ## Package name to root directory.
    deps Map[str, List[str]]  ## Package name to the dependencies its manifest declares.
    manifests List[pkg.ManifestFile]
    fn pkg.Packages.visit(mut self, pkg str, dir str, from pkg.Seen?, mut stack List[(str, str)], mut declared Map[str, pkg.Seen]) [env fs io mutates]  ## Read package `pkg`'s manifest in `dir` and its dependencies. `from` is the
    fn pkg.Packages.git_dir(mut self, me int, dep pkg.Dep) str? [mutates]  ## The directory of git dependency `dep`, fetched or from the cache (mod.pkg.7-8);
    fn pkg.Packages.package_of(self, module str) str  ## The package that module `module` belongs to: `""` for the root package's modules.
    pub fn pkg.Packages.target(self, from str, parts List[str]) pkg.Target  ## The module that the import path `parts`, written in module `from`, names
    pub fn pkg.Packages.finish(self, mut prog ast.Program, mut diags List[syntax.Diag], mut texts List[str]) [mutates]  ## Add the manifests to the program's files, after its source files, with
pub struct pkg.Target  ## What an import path written in some module names.
    pub module str  ## The module's qualified name in the program (`geo.shapes`).
    pub dir str
    pub undeclared bool  ## The import names a package that the importing package doesn't declare (mod.pkg.4).
struct pkg.Seen
    manifest int
    pos ast.Pos
fn pkg.is_dir(dir str) bool [fs io]  ## Whether `dir` is a directory.
fn pkg.canonical(dir str) str [env]  ## `dir` as an absolute path with `.` and `..` folded, for comparing two
pub fn pkg.read(root str) pkg.Packages [env fs io]  ## Read the manifest at `root`, if any, and every path dependency's.
```
