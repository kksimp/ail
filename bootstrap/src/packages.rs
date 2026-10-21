//! Package manifests, path dependencies and capability grants (mod.pkg.1-6,
//! eff.cap.4-5), ADRs 0230-0233.
//!
//! A package root may hold an `ail.pkg` manifest (mod.pkg.1): one line
//! `package NAME [caps LIST]`, then one `dep NAME path DIR [caps LIST]` or
//! `dep NAME git URL rev|tag PIN [caps LIST]` line per dependency
//! (mod.pkg.2-3, mod.pkg.6; git fetching and `ail.lock` are in `git.rs`). Every package in the dependency graph is
//! read before any source file. A dependency's modules are loaded under its
//! name (`use geo.shapes` is `DIR/shapes`), and a dependency resolves its own
//! imports against its own root and its own manifest (mod.pkg.3). The
//! bootstrap qualifies items by module name (ADR 0156), so package names are
//! global: two packages can't share a name, and the root package can't have
//! a module directory with a package's name (mod.pkg.4).
//!
//! Manifests become extra `SourceFile`s after the program's own, so their
//! diagnostics name `ail.pkg` and its line. `check_grants` runs after the
//! checker and reuses the effect inference of `--deny` (eff.cap.4-5).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{FILE_LINES, Grant, Program, SourceFile, is_prelude};
use crate::check::Resolved;
use crate::check::effects::{CAPABILITIES, Effects};
use crate::diag::{Diagnostic, Pos};

pub mod git;

/// The manifest's file name (mod.pkg.1).
pub const MANIFEST: &str = "ail.pkg";

/// Every package of a program: the root package (named `""`) and its
/// dependencies, transitively.
pub struct Packages {
    /// Package name to root directory.
    roots: HashMap<String, PathBuf>,
    /// Package name to the dependencies its manifest declares.
    deps: HashMap<String, Vec<String>>,
    manifests: Vec<ManifestFile>,
    /// `ail.lock` and the git dependencies resolved (mod.pkg.8).
    lock: git::Lock,
    opts: git::Options,
}

struct ManifestFile {
    package: String,
    path: PathBuf,
    src: String,
    /// Positions are local to the manifest until `finish`.
    diags: Vec<Diagnostic>,
    grants: Vec<Grant>,
    /// The packages this manifest lets contain `unsafe` code (unsafe.block.3).
    unsafe_ok: Vec<String>,
    /// The packages this manifest lets reach hardware: `unsafe driver` (unsafe.hw.1).
    driver_ok: Vec<String>,
}

/// What an import path written in some module names.
pub struct Target {
    /// The module's qualified name in the program (`geo.shapes`).
    pub module: String,
    pub dir: PathBuf,
    /// The import names a package that the importing package doesn't
    /// declare (mod.pkg.4).
    pub undeclared: bool,
}

impl Packages {
    /// Read the manifest at `root`, if any, and every path dependency's.
    pub fn read(root: &Path) -> Packages {
        let mut p = Packages {
            roots: HashMap::from([(String::new(), root.to_path_buf())]),
            deps: HashMap::new(),
            manifests: Vec::new(),
            lock: git::Lock::read(root),
            opts: git::options(),
        };
        let mut declared: HashMap<String, (usize, Pos)> = HashMap::new();
        let mut path = vec![(canonical(root), root.display().to_string())];
        p.visit("", root, None, &mut path, &mut declared);
        // mod.pkg.4: the root package's modules are unqualified, so none may
        // share a package's name.
        let mut names: Vec<(&String, &(usize, Pos))> = declared.iter().collect();
        names.sort_by_key(|(_, (m, pos))| (*m, pos.line));
        let mut clashes = Vec::new();
        for (name, &(m, pos)) in names {
            let dir = root.join(name);
            if dir.is_dir() {
                clashes.push((
                    m,
                    Diagnostic::new(
                        "mod.pkg.4",
                        pos,
                        format!("package `{name}` has the same name as the module directory `{}`", dir.display()),
                    )
                    .help("rename the directory, or the dependency on this line and in its own `ail.pkg`"),
                ));
            }
        }
        for (m, d) in clashes {
            p.manifests[m].diags.push(d);
        }
        p.settle_lock();
        p
    }

    /// mod.pkg.8: refuse a stale lock under `--deterministic`, else write it
    /// when the command writes locks.
    fn settle_lock(&mut self) {
        if !self.manifests.first().is_some_and(|m| m.package.is_empty()) {
            return;
        }
        if self.opts.frozen {
            let extra = self.lock.extra();
            if !extra.is_empty() {
                let names = extra.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ");
                self.manifests[0].diags.push(
                    Diagnostic::new(
                        "mod.pkg.8",
                        Pos { line: 1, col: 1 },
                        format!("`ail.lock` is stale: it locks {names}, which no `ail.pkg` declares"),
                    )
                    .help("run `ail pkg update` and commit `ail.lock`, or run without `--deterministic`"),
                );
            }
        } else if self.opts.write_lock || self.opts.refresh.is_some() {
            if let Err(e) = self.lock.save() {
                self.manifests[0].diags.push(Diagnostic::new("mod.pkg.8", Pos { line: 1, col: 1 }, e));
            }
        }
    }

    /// The git dependencies resolved, for `ail pkg update` (mod.pkg.9).
    pub fn locked(&self) -> &[git::Entry] {
        &self.lock.used
    }

    /// The directory of git dependency `dep`, fetched or from the cache
    /// (mod.pkg.7-8); `None` after a diagnostic in manifest `me`.
    fn git_dir(&mut self, me: usize, name: &str, source: &git::Source, pos: Pos) -> Option<PathBuf> {
        if let Some(e) = self.lock.used.iter().find(|e| e.name == name) {
            // Declared again by another manifest of the graph.
            if e.source == *source {
                return Some(git::home().join("deps").join(format!("{name}-{}", e.commit)));
            }
        }
        let locked = self.lock.locked(name, source, &self.opts.refresh).map(str::to_string);
        if locked.is_none() && self.opts.frozen {
            self.manifests[me].diags.push(
                Diagnostic::new(
                    "mod.pkg.8",
                    pos,
                    format!("`ail.lock` is stale: it doesn't lock dependency `{name}` as this line names it"),
                )
                .help(format!("run `ail pkg update {name}` and commit `ail.lock`, or run without `--deterministic`")),
            );
            self.lock.failed = true;
            return None;
        }
        match git::fetch(name, source, locked.as_deref(), self.opts.offline, pos) {
            Ok((commit, dir)) => {
                self.lock.used.push(git::Entry { name: name.to_string(), source: source.clone(), commit });
                Some(dir)
            }
            Err(d) => {
                self.manifests[me].diags.push(d);
                self.lock.failed = true;
                None
            }
        }
    }

    /// Read package `pkg`'s manifest in `dir` and its dependencies. `from`
    /// is the manifest and position of the `dep` line that declared it.
    fn visit(
        &mut self,
        pkg: &str,
        dir: &Path,
        from: Option<(usize, Pos)>,
        path: &mut Vec<(PathBuf, String)>,
        declared: &mut HashMap<String, (usize, Pos)>,
    ) {
        self.deps.insert(pkg.to_string(), Vec::new());
        let file = dir.join(MANIFEST);
        let Ok(src) = std::fs::read_to_string(&file) else { return };
        let (manifest, diags) = parse(&src);
        let me = self.manifests.len();
        self.manifests.push(ManifestFile {
            package: pkg.to_string(),
            path: file,
            src,
            diags,
            grants: Vec::new(),
            unsafe_ok: Vec::new(),
            driver_ok: Vec::new(),
        });
        if manifest.unsafe_ok {
            self.manifests[me].unsafe_ok.push(pkg.to_string());
        }
        if manifest.driver_ok {
            self.manifests[me].driver_ok.push(pkg.to_string());
        }
        if let (Some((name, _)), Some((parent, pos))) = (&manifest.name, from)
            && name != pkg
        {
            self.manifests[parent].diags.push(
                Diagnostic::new(
                    "mod.pkg.4",
                    pos,
                    format!("the package in `{}` is named `{name}`, not `{pkg}`", dir.display()),
                )
                .help(format!("declare it as `dep {name}`; `use {name} as {pkg}` renames it in one file")),
            );
        }
        if let (Some(caps), Some((_, pos))) = (manifest.caps, &manifest.name) {
            self.manifests[me].grants.push(Grant { package: pkg.to_string(), caps, pos: *pos, own: true });
        }
        let base = dir;
        for dep in manifest.deps {
            let dir = match (dep.path, &dep.git) {
                (Some(rel), _) => base.join(&rel),
                (None, Some(source)) => match self.git_dir(me, &dep.name, source, dep.pos) {
                    Some(dir) => dir,
                    None => continue,
                },
                (None, None) => continue,
            };
            if !dir.is_dir() {
                self.manifests[me].diags.push(
                    Diagnostic::new("mod.pkg.3", dep.pos, format!("there is no directory `{}`", dir.display()))
                        .help("a `path` is relative to the directory that holds this `ail.pkg`"),
                );
                continue;
            }
            let canon = canonical(&dir);
            if let Some(at) = path.iter().position(|(d, _)| *d == canon) {
                let mut cycle: Vec<String> = path[at..].iter().map(|(_, shown)| shown.clone()).collect();
                cycle.push(dir.display().to_string());
                self.manifests[me].diags.push(
                    Diagnostic::new(
                        "mod.pkg.5",
                        dep.pos,
                        format!("package dependencies form a cycle: {}", cycle.join(" -> ")),
                    )
                    .help("move what the packages share into a package that both depend on"),
                );
                continue;
            }
            self.deps.get_mut(pkg).expect("visited").push(dep.name.clone());
            if dep.unsafe_ok {
                self.manifests[me].unsafe_ok.push(dep.name.clone());
            }
            if dep.driver_ok {
                self.manifests[me].driver_ok.push(dep.name.clone());
            }
            self.manifests[me].grants.push(Grant {
                package: dep.name.clone(),
                caps: dep.caps,
                pos: dep.pos,
                own: false,
            });
            match self.roots.get(&dep.name) {
                Some(seen) if canonical(seen) == canon => {}
                Some(seen) => {
                    self.manifests[me].diags.push(
                        Diagnostic::new(
                            "mod.pkg.4",
                            dep.pos,
                            format!(
                                "two packages are named `{}`: `{}` and `{}`",
                                dep.name,
                                seen.display(),
                                dir.display()
                            ),
                        )
                        .help("the packages of one program need different names"),
                    );
                }
                None => {
                    self.roots.insert(dep.name.clone(), dir.clone());
                    declared.insert(dep.name.clone(), (me, dep.pos));
                    path.push((canon, dir.display().to_string()));
                    self.visit(&dep.name, &dir, Some((me, dep.pos)), path, declared);
                    path.pop();
                }
            }
        }
    }

    /// The package that module `module` belongs to: `""` for the root
    /// package's modules.
    pub fn package_of<'a>(&self, module: &'a str) -> &'a str {
        let head = module.split('.').next().unwrap_or("");
        if !head.is_empty() && self.roots.contains_key(head) { head } else { "" }
    }

    /// The module that the import path `path`, written in module `from`,
    /// names (mod.unit.2, mod.pkg.3).
    pub fn target(&self, from: &str, path: &[String]) -> Target {
        let pkg = self.package_of(from);
        let head = path[0].as_str();
        let dotted = path.join(".");
        if self.deps.get(pkg).is_some_and(|d| d.iter().any(|n| n == head))
            && let Some(dir) = self.roots.get(head)
        {
            let dir = path[1..].iter().fold(dir.clone(), |d, s| d.join(s));
            return Target { module: dotted, dir, undeclared: false };
        }
        let dir = path.iter().fold(self.roots[pkg].clone(), |d, s| d.join(s));
        let module = if pkg.is_empty() { dotted } else { format!("{pkg}.{dotted}") };
        Target { module, dir, undeclared: head != pkg && self.roots.contains_key(head) }
    }

    /// Add the manifests to the program's files, after its source files,
    /// with their diagnostics and grants.
    pub fn finish(self, prog: &mut Program, diags: &mut Vec<Diagnostic>) {
        for m in self.manifests {
            let base = prog.files.len() * FILE_LINES;
            let at = |p: Pos| Pos { line: p.line + base, col: p.col };
            for mut d in m.diags {
                d.pos = at(d.pos);
                d.end = d.end.map(at);
                for n in d.notes.iter_mut() {
                    n.pos = at(n.pos);
                }
                diags.push(d);
            }
            prog.grants.extend(m.grants.into_iter().map(|g| Grant { pos: at(g.pos), ..g }));
            prog.unsafe_allowed.get_or_insert_with(Vec::new).extend(m.unsafe_ok);
            prog.driver_allowed.extend(m.driver_ok);
            prog.files.push(SourceFile {
                path: m.path.display().to_string(),
                src: m.src,
                module: m.package,
                base,
            });
        }
    }
}

fn canonical(dir: &Path) -> PathBuf {
    dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf())
}

/// A parsed manifest.
#[derive(Default)]
struct Manifest {
    name: Option<(String, Pos)>,
    caps: Option<Vec<&'static str>>,
    /// `unsafe allow` on the `package` line (unsafe.block.3).
    unsafe_ok: bool,
    /// `unsafe driver` on the `package` line (unsafe.hw.1).
    driver_ok: bool,
    deps: Vec<Dep>,
}

struct Dep {
    name: String,
    pos: Pos,
    /// `path DIR`; `None` for a git dependency or an error.
    path: Option<String>,
    /// `git URL` with its pin (mod.pkg.6).
    git: Option<git::Source>,
    caps: Vec<&'static str>,
    /// `unsafe allow` on the `dep` line (unsafe.block.3).
    unsafe_ok: bool,
    /// `unsafe driver` on the `dep` line (unsafe.hw.1).
    driver_ok: bool,
}

/// The words of one line, with their columns: whitespace separates them,
/// `"..."` quotes one, and `#` starts a comment (mod.pkg.2).
pub(crate) fn words(line: &str, n: usize, diags: &mut Vec<Diagnostic>) -> Vec<(String, Pos)> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }
        let pos = Pos { line: n, col: i + 1 };
        if chars[i] == '#' {
            break;
        }
        if chars[i] == '"' {
            let Some(len) = chars[i + 1..].iter().position(|&c| c == '"') else {
                diags.push(Diagnostic::new("mod.pkg.2", pos, "this quoted word has no closing `\"`"));
                break;
            };
            out.push((chars[i + 1..i + 1 + len].iter().collect(), pos));
            i += len + 2;
            continue;
        }
        let start = i;
        while i < chars.len() && !chars[i].is_whitespace() {
            i += 1;
        }
        out.push((chars[start..i].iter().collect(), pos));
    }
    out
}

fn is_name(s: &str) -> bool {
    let mut cs = s.chars();
    cs.next().is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && cs.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// `fs,env`, `all` or `none` (eff.cap.2).
fn caps(list: &str, pos: Pos, diags: &mut Vec<Diagnostic>) -> Vec<&'static str> {
    if list == "none" {
        return Vec::new();
    }
    let mut out = Vec::new();
    for name in list.split(',') {
        if name == "all" {
            out.extend_from_slice(CAPABILITIES);
        } else if let Some(&cap) = CAPABILITIES.iter().find(|c| **c == name) {
            out.push(cap);
        } else {
            diags.push(
                Diagnostic::new("eff.cap.2", pos, format!("`{name}` is not a capability"))
                    .help(format!("`caps` takes a comma-separated list of {}, or `all` or `none`", CAPABILITIES.join(", "))),
            );
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Parse a manifest (mod.pkg.2-3, mod.pkg.6). Positions are local.
fn parse(src: &str) -> (Manifest, Vec<Diagnostic>) {
    let mut m = Manifest::default();
    let mut diags = Vec::new();
    let mut names: HashSet<String> = HashSet::new();
    for (i, line) in src.lines().enumerate() {
        let ws = words(line, i + 1, &mut diags);
        let Some(((kw, kw_pos), rest)) = ws.split_first() else { continue };
        if m.name.is_none() && kw != "package" {
            diags.push(
                Diagnostic::new("mod.pkg.2", *kw_pos, "an `ail.pkg` starts with `package NAME`")
                    .help("add a first line such as `package app`"),
            );
            m.name = Some((String::new(), *kw_pos));
        }
        let allowed: &[&str] = match kw.as_str() {
            "package" => &["caps", "unsafe"],
            "dep" => &["path", "git", "rev", "tag", "caps", "unsafe"],
            _ => {
                diags.push(
                    Diagnostic::new("mod.pkg.2", *kw_pos, format!("`{kw}` is not a manifest line"))
                        .help("a line is `package NAME ...` or `dep NAME ...`"),
                );
                continue;
            }
        };
        let Some(((name, name_pos), pairs)) = rest.split_first() else {
            diags.push(
                Diagnostic::new("mod.pkg.2", *kw_pos, format!("`{kw}` needs a name"))
                    .help(format!("write `{kw} NAME ...`")),
            );
            continue;
        };
        if !is_name(name) {
            diags.push(
                Diagnostic::new("mod.pkg.2", *name_pos, format!("`{name}` is not a package name"))
                    .help("a package name is lowercase letters, digits and `_`, like a module directory"),
            );
        }
        let mut keys: HashMap<&str, (&str, Pos)> = HashMap::new();
        for pair in pairs.chunks(2) {
            let (key, kpos) = (&pair[0].0, pair[0].1);
            let Some((value, _)) = pair.get(1) else {
                diags.push(
                    Diagnostic::new("mod.pkg.2", kpos, format!("`{key}` needs a value"))
                        .help(format!("write `{key} VALUE`")),
                );
                continue;
            };
            if !allowed.contains(&key.as_str()) {
                diags.push(
                    Diagnostic::new("mod.pkg.2", kpos, format!("`{key}` is not a key of a `{kw}` line"))
                        .help(format!("the keys are {}", allowed.join(", "))),
                );
            } else if keys.insert(key, (value, kpos)).is_some() {
                diags.push(Diagnostic::new("mod.pkg.2", kpos, format!("`{key}` is given twice on this line")));
            }
        }
        // unsafe.block.3, unsafe.hw.1: `driver` allows everything `allow` does, and hardware access.
        let (unsafe_ok, driver_ok) = match keys.get("unsafe") {
            Some(("allow", _)) => (true, false),
            Some(("driver", _)) => (true, true),
            Some((v, p)) => {
                diags.push(
                    Diagnostic::new("unsafe.block.3", *p, format!("`unsafe {v}` is not an unsafe setting"))
                        .help("write `unsafe allow` to let this package contain `unsafe` code, or `unsafe driver` to let it reach hardware as well"),
                );
                (false, false)
            }
            None => (false, false),
        };
        let grant = keys.get("caps").map(|(v, p)| caps(v, *p, &mut diags));
        if kw == "package" {
            if m.name.as_ref().is_some_and(|(n, _)| !n.is_empty()) {
                diags.push(Diagnostic::new("mod.pkg.2", *kw_pos, "an `ail.pkg` has one `package` line"));
                continue;
            }
            m.name = Some((name.clone(), *name_pos));
            m.caps = grant;
            m.unsafe_ok = unsafe_ok;
            m.driver_ok = driver_ok;
            continue;
        }
        if !names.insert(name.clone()) || name == "std" {
            let msg = if name == "std" {
                "`std` is the standard library, not a package name".to_string()
            } else {
                format!("`{name}` is declared twice")
            };
            diags.push(Diagnostic::new("mod.pkg.4", *name_pos, msg).help("give each dependency its own name"));
            continue;
        }
        let mut git = None;
        let path = match (keys.get("path"), keys.get("git")) {
            (Some((p, _)), None) => {
                for k in ["rev", "tag"] {
                    if let Some((_, kp)) = keys.get(k) {
                        diags.push(Diagnostic::new("mod.pkg.6", *kp, format!("`{k}` pins a `git` dependency, not a `path` one")));
                    }
                }
                Some(p.to_string())
            }
            (None, Some((url, gpos))) => {
                match (keys.get("rev"), keys.get("tag")) {
                    (Some((pin, _)), None) => git = Some(git::Source { url: url.to_string(), kind: "rev", pin: pin.to_string() }),
                    (None, Some((pin, _))) => git = Some(git::Source { url: url.to_string(), kind: "tag", pin: pin.to_string() }),
                    _ => diags.push(
                        Diagnostic::new("mod.pkg.6", *gpos, "a `git` dependency needs exactly one of `rev` and `tag`")
                            .help("pin it with `rev COMMIT` or `tag TAG`"),
                    ),
                }
                None
            }
            (Some(_), Some((_, gpos))) => {
                diags.push(Diagnostic::new("mod.pkg.3", *gpos, "a dependency has either `path` or `git`, not both"));
                None
            }
            (None, None) => {
                diags.push(
                    Diagnostic::new("mod.pkg.3", *name_pos, format!("dependency `{name}` has no `path` or `git`"))
                        .help(format!("write `dep {name} path DIR`")),
                );
                None
            }
        };
        m.deps.push(Dep { name: name.clone(), pos: *name_pos, path, git, caps: grant.unwrap_or_default(), unsafe_ok, driver_ok });
    }
    if m.name.is_none() {
        diags.push(
            Diagnostic::new("mod.pkg.2", Pos { line: 1, col: 1 }, "an `ail.pkg` starts with `package NAME`")
                .help("add a first line such as `package app`"),
        );
    }
    (m, diags)
}

/// eff.cap.4-5: the code of each package reaches only the capabilities its
/// grants allow. One diagnostic per grant and capability, at the manifest
/// line, naming a chain of calls from a function of the package (a public
/// one if any reaches it).
pub fn check_grants(prog: &Program, resolved: &Resolved) -> Vec<Diagnostic> {
    if prog.grants.is_empty() {
        return Vec::new();
    }
    let fx = Effects::infer(prog, resolved);
    let deps: HashSet<&str> = prog.grants.iter().filter(|g| !g.own).map(|g| g.package.as_str()).collect();
    let package_of = |module: &str| {
        let head = module.split('.').next().unwrap_or("");
        if deps.contains(head) { head.to_string() } else { String::new() }
    };
    let owner: Vec<Option<String>> = prog
        .fns
        .iter()
        .map(|f| (!is_prelude(f.pos)).then(|| package_of(prog.module_at(f.pos))))
        .collect();
    let mut diags = Vec::new();
    for g in &prog.grants {
        for &cap in CAPABILITIES.iter().filter(|c| !g.caps.contains(c)) {
            let reaching = |i: &usize| owner[*i].as_deref() == Some(g.package.as_str()) && fx.of(*i).contains(cap);
            let Some(i) = (0..prog.fns.len())
                .filter(reaching)
                .find(|&i| prog.is_pub(prog.fns[i].pos))
                .or_else(|| (0..prog.fns.len()).find(reaching))
            else {
                continue;
            };
            let chain = fx.chain(i, cap);
            let (rule, who, line) = if g.own {
                ("eff.cap.5", "this package".to_string(), "its `package` line")
            } else {
                ("eff.cap.4", format!("dependency `{}`", g.package), "this `dep` line")
            };
            let help = if g.caps.is_empty() {
                format!("add `caps {cap}` to {line} if the package should reach `{cap}`; otherwise remove the call")
            } else {
                format!(
                    "add `{cap}` to the `caps` of {line} if the package should reach it; otherwise remove the call"
                )
            };
            let first = chain.split(" -> ").next().unwrap_or("");
            diags.push(
                Diagnostic::new(rule, g.pos, format!("{who} reaches the capability `{cap}`, which {line} doesn't grant: {chain}"))
                    .note(prog.fns[i].pos, format!("`{first}` reaches `{cap}`"))
                    .help(help),
            );
        }
    }
    diags
}

/// `ail pkg update [NAME...]` (mod.pkg.9): resolve the named git
/// dependencies of the package in the current directory (all if none are
/// named) again, ignoring `ail.lock`, and rewrite the lock.
pub fn update_command(names: &[&str], json: bool) -> std::process::ExitCode {
    let refresh: Vec<String> = names.iter().map(|n| n.to_string()).collect();
    let o = git::options();
    git::configure(git::Options { write_lock: true, frozen: false, refresh: Some(refresh.clone()), ..o });
    let root = PathBuf::from(".");
    let packages = Packages::read(&root);
    let mut diags = Vec::new();
    for n in &refresh {
        if !packages.locked().iter().any(|e| e.name == *n) && packages.manifests.first().is_some_and(|m| m.package.is_empty()) {
            diags.push(
                Diagnostic::new("mod.pkg.9", Pos { line: 1, col: 1 }, format!("no `ail.pkg` of this program has a git dependency `{n}`"))
                    .help("name a `dep NAME git ...` line of `ail.pkg` or of a dependency's"),
            );
        }
    }
    if packages.manifests.is_empty() {
        eprintln!("error: there is no `{MANIFEST}` in the current directory (mod.pkg.9)");
        return std::process::ExitCode::from(2);
    }
    let mut entries: Vec<String> = packages.locked().iter().map(|e| format!("{} {}", e.name, e.commit)).collect();
    entries.sort();
    let mut prog = Program::default();
    let mut local = Vec::new();
    // Diagnostics from `read` are local to their manifest; `finish` shifts them.
    packages.finish(&mut prog, &mut local);
    let at = Pos { line: prog.files.first().map_or(0, |f| f.base) + 1, col: 1 };
    local.extend(diags.into_iter().map(|d| Diagnostic { pos: at, ..d }));
    if !local.is_empty() {
        crate::modules::report(&prog.files, local, json);
        return std::process::ExitCode::from(2);
    }
    for e in entries {
        println!("locked {e}");
    }
    std::process::ExitCode::SUCCESS
}
