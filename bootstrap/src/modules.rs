//! Packages and modules (ADRs 0097-0099, 0153-0156).
//!
//! `ail run DIR` makes DIR's `.ail` files the root module; `ail run FILE`
//! makes FILE alone the root module (mod.unit.3). Either way the package root
//! is that directory, and `use a.b` loads every `.ail` file directly in
//! `ROOT/a/b` as module `a.b` (mod.unit.1-2, mod.unit.5), lazily and
//! transitively.
//!
//! All files are parsed as one program. Each file's lines are numbered from
//! a multiple of `ast::FILE_LINES`, so a position tells its file and module.
//! The items of module `a.b` are then renamed to their qualified names
//! (`a.b.Point`), and every reference is rewritten to that name: bare names
//! inside the module, and `b.Point`, `b.f(...)`, `b.E.V` through an import
//! (mod.use.1-2). The checker and interpreter then see one program whose
//! names are unique. Imports (mod.use.3-6) and access to private items
//! (mod.vis.1) are checked here; private fields and methods, which need
//! types, in the checker (`Program::hidden`).

use crate::fasthash::{FastMap as HashMap, FastSet as HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{
    Block, Cond, Else, Expr, FILE_LINES, GenericParam, InterpPart, Pattern, Program, SourceFile, Stmt, TypeKind,
    TypeRef, Within, is_prelude,
};
use crate::diag::{Diagnostic, Pos};
use crate::lexer::{Part, Tok, Token, lex};

/// Read the program at `arg` (a file or a package directory) with every
/// module it imports. `Err` is a message for a file or directory that
/// can't be read.
pub fn load(arg: &str) -> Result<(Program, Vec<Diagnostic>), String> {
    let path = Path::new(arg);
    let (root, root_files) = if path.is_dir() {
        let files = ail_files(path);
        if files.is_empty() {
            return Err(format!("{arg} has no .ail files (mod.unit.5)"));
        }
        (path.to_path_buf(), files)
    } else {
        (parent(path), vec![path.to_path_buf()])
    };
    load_package(root, root_files, None)
}

/// Like `load` for the package directory `dir`, for a query about `target`
/// (text `src`) in it (api.pkg.2). Also returns how many lines to add to a
/// line of `target` to get a position in the program: 0 for a file of the
/// root module, which is loaded first so nothing shifts, else its file base.
pub fn load_in(dir: &str, target: &str, src: &str) -> Result<(Program, Vec<Diagnostic>, usize), String> {
    let mut files = ail_files(Path::new(dir));
    let want = std::fs::canonicalize(target).map_err(|e| format!("can't read {target}: {e}"))?;
    let same = |f: &PathBuf| std::fs::canonicalize(f).is_ok_and(|c| c == want);
    if let Some(at) = files.iter().position(same) {
        let first = files.remove(at);
        files.insert(0, first);
        let (prog, diags) = load_package(PathBuf::from(dir), files, Some(src.to_string()))?;
        return Ok((prog, diags, 0));
    }
    let (prog, diags) = load(dir)?;
    let base = prog.files.iter().find(|f| same(&PathBuf::from(&f.path))).map(|f| f.base).ok_or_else(|| {
        format!("{target} is not part of the package {dir}: no root file imports its module (api.pkg.2)")
    })?;
    Ok((prog, diags, base))
}

/// Like `load` for a single file whose text is `src` (for `ail api`).
pub fn load_src(path: &str, src: &str) -> (Program, Vec<Diagnostic>) {
    let path = Path::new(path);
    load_package(parent(path), vec![path.to_path_buf()], Some(src.to_string()))
        .unwrap_or_else(|e| (Program::default(), vec![Diagnostic::new("mod.use.4", Pos { line: 1, col: 1 }, e)]))
}

fn parent(path: &Path) -> PathBuf {
    path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf()
}

/// Load the root module's files, the first given as `first_src` if any,
/// and every module they import.
fn load_package(
    root: PathBuf,
    root_files: Vec<PathBuf>,
    mut first_src: Option<String>,
) -> Result<(Program, Vec<Diagnostic>), String> {
    let mut files: Vec<SourceFile> = Vec::new();
    let mut tokens: Vec<Token> = Vec::new();
    let mut diags = Vec::new();
    let mut seen: HashSet<String> = HashSet::from_iter([String::new()]);
    let mut queue: Vec<(String, Vec<PathBuf>)> = vec![(String::new(), root_files)];
    // The manifests come first: they say where each package is (mod.pkg.1-3).
    let packages = crate::packages::Packages::read(&root);
    while let Some((module, paths)) = queue.pop() {
        for p in paths {
            // Positions from PRELUDE_LINE up belong to the prelude (ADR 0156).
            if (files.len() + 1) * FILE_LINES >= crate::ast::PRELUDE_LINE {
                return Err(format!("the program has more than {} files", files.len()));
            }
            let src = match first_src.take() {
                Some(src) => src,
                None => match crate::stdlib::ail_module_source(&p.display().to_string()) {
                    Some(src) => src.to_string(),
                    None => std::fs::read_to_string(&p).map_err(|e| format!("can't read {}: {e}", p.display()))?,
                },
            };
            let base = files.len() * FILE_LINES;
            let (mut toks, lex_diags) = lex(&src);
            toks.pop(); // Eof
            shift(&mut toks, base);
            diags.extend(lex_diags.into_iter().map(|mut d| {
                d.pos.line += base;
                for e in d.fixes.iter_mut().flat_map(|f| f.edits.iter_mut()) {
                    e.pos.line += base;
                    e.end.line += base;
                }
                d
            }));
            for import in scan_uses(&toks) {
                if import.first().is_some_and(|s| s == "std") {
                    // std.ui.19: a standard module written in AIL.
                    let dotted = import.join(".");
                    if let Some(path) = crate::stdlib::ail_module_path(&dotted)
                        && seen.insert(dotted.clone())
                    {
                        queue.push((dotted, vec![PathBuf::from(path)]));
                    }
                    continue;
                }
                let target = packages.target(&module, &import);
                if !target.undeclared && seen.insert(target.module.clone()) {
                    let found = if target.dir.is_dir() { ail_files(&target.dir) } else { Vec::new() };
                    if !found.is_empty() {
                        queue.push((target.module, found));
                    }
                }
            }
            tokens.extend(toks);
            files.push(SourceFile { path: p.display().to_string(), src, module: module.clone(), base });
        }
    }
    let end = Pos { line: files.len() * FILE_LINES, col: 1 };
    tokens.push(Token { tok: Tok::Eof, pos: end });

    // ct.derive.2: a derive generator needs the parsed program, and its
    // output is parsed again with it (`derive.rs`).
    let again = (diags.is_empty() && crate::derive::wanted(&tokens)).then(|| (tokens.clone(), files.clone()));
    let (mut prog, mut diags) = assemble(tokens, files, &packages, diags);
    if let Some((tokens, files)) = again
        && !diags.iter().any(|d| !d.warning)
    {
        (prog, diags) = crate::derive::expand(prog, diags, tokens, files, &|t, f| assemble(t, f, &packages, Vec::new()));
    }
    packages.finish(&mut prog, &mut diags);
    Ok((prog, diags))
}

/// Parse the tokens of `files` after the prelude and resolve their names.
fn assemble(
    tokens: Vec<Token>,
    files: Vec<SourceFile>,
    packages: &crate::packages::Packages,
    mut diags: Vec<Diagnostic>,
) -> (Program, Vec<Diagnostic>) {
    let (mut prog, parse_diags) = crate::parser::parse_after_prelude(tokens);
    diags.extend(parse_diags);
    one_per_line(&mut diags);
    prog.files = files;
    resolve(&mut prog, &mut diags, packages);
    (prog, diags)
}

/// The `.ail` files directly in `dir`, sorted (mod.unit.1).
fn ail_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> =
        std::fs::read_dir(dir).map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).collect()).unwrap_or_default();
    files.retain(|p| p.is_file() && p.extension().is_some_and(|x| x == "ail"));
    files.sort();
    files
}

/// One syntax diagnostic per line (gram.rec.1, ADR 0263): a mistake on a line
/// often breaks the lexer and the parser in turn, and the first report, the
/// leftmost, names it. Fixes of the reports it absorbs join its own when they
/// don't overlap, so `ail fix` still settles the whole line in one round.
/// String errors stay: they are independent of the rest.
fn one_per_line(diags: &mut Vec<Diagnostic>) {
    diags.sort_by_key(|d| (d.pos.line, d.pos.col));
    let mut kept: Vec<Diagnostic> = Vec::new();
    for d in std::mem::take(diags) {
        let same_line = kept.last().is_some_and(|k| !k.warning && k.pos.line == d.pos.line);
        if d.warning || d.rule.starts_with("lex.str.") || !same_line {
            kept.push(d);
            continue;
        }
        let last = kept.last_mut().unwrap();
        if let (Some(into), Some(from)) = (last.fixes.first_mut(), d.fixes.first())
            && explicit(&into.edits)
            && explicit(&from.edits)
            && !from.edits.iter().any(|e| into.edits.iter().any(|o| overlaps(o, e)))
        {
            let more: Vec<_> = d.fixes.into_iter().next().unwrap().edits;
            into.edits.extend(more);
        }
    }
    *diags = kept;
}

fn explicit(edits: &[crate::diag::Edit]) -> bool {
    edits.iter().all(|e| e.old.is_none() && e.anchor == crate::diag::Anchor::Exact)
}

fn overlaps(a: &crate::diag::Edit, b: &crate::diag::Edit) -> bool {
    let key = |p: Pos| (p.line, p.col);
    key(a.pos) <= key(b.end) && key(b.pos) <= key(a.end)
}

fn shift(tokens: &mut [Token], base: usize) {
    for t in tokens {
        t.pos.line += base;
        if let Tok::Interp(parts) = &mut t.tok {
            for p in parts {
                if let Part::Code(code) | Part::Fmt(code, _) = p {
                    shift(code, base);
                }
            }
        }
    }
}

/// The module paths of a file's top-level `use` lines, to load them before
/// parsing.
fn scan_uses(tokens: &[Token]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    let mut depth = 0usize;
    for (i, t) in tokens.iter().enumerate() {
        match t.tok {
            Tok::Indent => depth += 1,
            Tok::Dedent => depth = depth.saturating_sub(1),
            Tok::Kw("use") if depth == 0 => {
                let mut path = Vec::new();
                for t in &tokens[i + 1..] {
                    match &t.tok {
                        Tok::Ident(s) => path.push(s.clone()),
                        Tok::Sym(".") => {}
                        _ => break,
                    }
                }
                out.push(path);
            }
            _ => {}
        }
    }
    out
}

/// What an import name refers to.
#[derive(Clone)]
enum Target {
    Module(String),
    Std,
}

/// The top-level items of every module, by module path and name.
type Items = HashMap<String, HashMap<String, Pos>>;

fn qualified(module: &str, name: &str) -> String {
    if module.is_empty() { name.to_string() } else { format!("{module}.{name}") }
}

fn file_index(prog: &Program, pos: Pos) -> Option<usize> {
    if is_prelude(pos) {
        return None;
    }
    prog.files.iter().rposition(|f| f.base < pos.line)
}

fn resolve(prog: &mut Program, diags: &mut Vec<Diagnostic>, packages: &crate::packages::Packages) {
    let modules: Vec<String> = {
        let mut m: Vec<String> = prog.files.iter().map(|f| f.module.clone()).collect();
        m.dedup();
        m
    };
    let module_of = |prog: &Program, pos: Pos| prog.module_at(pos).to_string();

    // Every module's top-level names (mod.unit.1), before renaming.
    let mut items: Items = modules.iter().map(|m| (m.clone(), HashMap::default())).collect();
    {
        let mut add = |name: &str, pos: Pos| {
            if is_prelude(pos) {
                return;
            }
            let m = module_of(prog, pos);
            items.entry(m).or_default().entry(name.to_string()).or_insert(pos);
        };
        for f in prog.fns.iter().filter(|f| f.top_level() && f.owner.is_none()) {
            add(&f.name, f.pos);
        }
        for s in &prog.structs {
            add(&s.name, s.pos);
        }
        for e in &prog.enums {
            add(&e.name, e.pos);
        }
        for t in &prog.traits {
            add(&t.name, t.pos);
        }
        for c in &prog.consts {
            add(&c.name, c.pos);
        }
    }

    // Imports, per file (mod.use.1, mod.use.3-6).
    let mut imports: Vec<HashMap<String, Target>> = vec![HashMap::default(); prog.files.len()];
    let mut first_decl: Vec<usize> = vec![usize::MAX; prog.files.len()];
    {
        let decls = prog
            .fns
            .iter()
            .filter(|f| f.top_level())
            .map(|f| f.pos)
            .chain(prog.structs.iter().map(|s| s.pos))
            .chain(prog.enums.iter().map(|e| e.pos))
            .chain(prog.traits.iter().map(|t| t.pos))
            .chain(prog.impls.iter().map(|i| i.pos))
            .chain(prog.consts.iter().map(|c| c.pos));
        for pos in decls {
            if let Some(fi) = file_index(prog, pos) {
                first_decl[fi] = first_decl[fi].min(pos.line);
            }
        }
    }
    let mut edges: Vec<(String, String, Pos)> = Vec::new();
    let mut unresolved: Vec<HashSet<String>> = vec![HashSet::default(); prog.files.len()];
    for u in &prog.uses {
        let Some(fi) = file_index(prog, u.pos) else { continue };
        let module = prog.files[fi].module.clone();
        let dotted = u.dotted();
        let (name, name_pos) = (u.name().0.as_str(), u.name().1);
        if u.pos.line > first_decl[fi] {
            diags.push(
                Diagnostic::new("mod.use.3", u.pos, "`use` must come before the file's declarations")
                    .help("move the import to the top of the file"),
            );
        }
        // The standard modules, and their errors (std.use.1-3), are the
        // checker's (`check/stdlib.rs`); their uses are left as written.
        let path: Vec<String> = u.path.iter().map(|(s, _)| s.clone()).collect();
        let found = packages.target(&module, &path);
        let target = if u.path[0].0 == "std" && crate::stdlib::ail_module_path(&dotted).is_some() {
            Some(Target::Module(dotted.clone()))
        } else if u.path[0].0 == "std" {
            Some(Target::Std)
        } else if found.module == module {
            diags.push(
                Diagnostic::new("mod.use.5", u.path[0].1, format!("module `{dotted}` imports itself"))
                    .help("remove the import; a module's own items are used without one"),
            );
            None
        } else if found.undeclared {
            let head = &u.path[0].0;
            diags.push(
                Diagnostic::new(
                    "mod.pkg.3",
                    u.path[0].1,
                    format!("package `{head}` is not a dependency of this package"),
                )
                .help(format!("add `dep {head} path DIR` to this package's `ail.pkg`")),
            );
            None
        } else if items.contains_key(&found.module) && !found.module.is_empty() {
            edges.push((module.clone(), found.module.clone(), u.path[0].1));
            Some(Target::Module(found.module))
        } else {
            let std_names = crate::stdlib::module_names();
            let std_help = crate::check::suggest(&dotted, &std_names)
                .map(|s| format!("did you mean `{s}`? The standard module `std.{s}` needs no `use` (std.use.4)"));
            diags.push(
                Diagnostic::new("mod.use.4", u.path[0].1, format!("there is no module `{dotted}`"))
                    .help(std_help.unwrap_or_else(|| {
                        "a module is a directory of `.ail` files; the standard modules need no `use` and are named `regex`, `fs`, `net`, `time` and so on (std.use.4)".to_string()
                    })),
            );
            None
        };
        if let Some(first) = imports[fi].get(name) {
            if matches!((first, &target), (Target::Std, Some(Target::Std))) && !unresolved[fi].contains(name) {
                continue; // std.use.3
            }
            diags.push(
                Diagnostic::new("mod.use.6", name_pos, format!("`{name}` is already imported in this file"))
                    .help(format!("write `use {dotted} as OTHER` to give it another name")),
            );
            continue;
        }
        if items.get(&module).is_some_and(|m| m.contains_key(name)) {
            diags.push(
                Diagnostic::new(
                    "mod.use.6",
                    name_pos,
                    format!("`{name}` is already the name of an item in this module"),
                )
                .help(format!("write `use {dotted} as OTHER`, or rename the item")),
            );
            continue;
        }
        // An unresolved import is still bound, so its uses aren't reported again.
        if target.is_none() {
            unresolved[fi].insert(name.to_string());
        }
        imports[fi].insert(name.to_string(), target.unwrap_or(Target::Std));
    }
    report_cycles(&edges, diags);

    // Rename each module's declarations to their qualified names, so that
    // implicit `self` sees each type's own members (trait.method.6).
    let generics_of = |gs: &[GenericParam]| gs.iter().map(|g| g.name.clone()).collect::<Vec<_>>();
    let public = prog.public.clone();
    let files = prog.files.clone();
    let cx_for = |pos: Pos, generics: Vec<String>| -> Option<Cx<'_>> {
        let fi = files.iter().rposition(|f| f.base < pos.line).filter(|_| !is_prelude(pos))?;
        Some(Cx {
            module: files[fi].module.clone(),
            items: &items,
            imports: &imports[fi],
            public: &public,
            generics,
            diags: Vec::new(),
        })
    };
    for s in prog.structs.iter_mut() {
        if let Some(cx) = cx_for(s.pos, Vec::new()) {
            s.name = qualified(&cx.module, &s.name);
        }
    }
    for e in prog.enums.iter_mut() {
        if let Some(cx) = cx_for(e.pos, Vec::new()) {
            e.name = qualified(&cx.module, &e.name);
        }
    }
    for t in prog.traits.iter_mut() {
        if let Some(cx) = cx_for(t.pos, Vec::new()) {
            t.name = qualified(&cx.module, &t.name);
        }
    }
    for c in prog.consts.iter_mut() {
        if let Some(cx) = cx_for(c.pos, Vec::new()) {
            c.name = qualified(&cx.module, &c.name);
        }
    }
    for i in 0..prog.impls.len() {
        let imp = &mut prog.impls[i];
        let gens = imp.target.params.as_deref().map(generics_of).unwrap_or_default();
        if let Some(mut cx) = cx_for(imp.pos, gens) {
            imp.trait_name = cx.type_name(&imp.trait_name, imp.trait_pos);
            imp.target.name = cx.type_name(&imp.target.name, imp.target.pos);
            diags.extend(cx.diags);
        }
    }
    for f in prog.fns.iter_mut() {
        let Some(mut cx) = cx_for(f.pos, Vec::new()) else { continue };
        match (&mut f.owner, f.within) {
            (None, Within::TopLevel) => f.name = qualified(&cx.module, &f.name),
            // The parser rejects `fn m.T.f` (mod.unit.4), so the owner is local.
            (Some(o), Within::TopLevel) => o.name = cx.type_name(&o.name, o.pos),
            _ => {}
        }
        diags.extend(cx.diags);
    }

    crate::implicit_self::rewrite(prog, diags);

    // Rewrite every reference to an item to its qualified name.
    for s in prog.structs.iter_mut() {
        if let Some(mut cx) = cx_for(s.pos, generics_of(&s.generics)) {
            cx.generic_bounds(&mut s.generics);
            for f in s.fields.iter_mut() {
                cx.type_ref(&mut f.ty);
            }
            for c in s.invariants.iter_mut() {
                cx.expr(&mut c.cond);
            }
            diags.extend(cx.diags);
        }
    }
    for e in prog.enums.iter_mut() {
        if let Some(mut cx) = cx_for(e.pos, generics_of(&e.generics)) {
            cx.generic_bounds(&mut e.generics);
            for f in e.variants.iter_mut().flat_map(|v| v.fields.iter_mut()) {
                cx.type_ref(&mut f.ty);
            }
            diags.extend(cx.diags);
        }
    }
    for c in prog.consts.iter_mut() {
        if let Some(mut cx) = cx_for(c.pos, Vec::new()) {
            cx.expr(&mut c.value);
            diags.extend(cx.diags);
        }
    }
    let impl_generics: Vec<Vec<String>> =
        prog.impls.iter().map(|i| i.target.params.as_deref().map(generics_of).unwrap_or_default()).collect();
    for f in prog.fns.iter_mut() {
        let mut gens = generics_of(&f.generics);
        if let Some(o) = &f.owner {
            gens.extend(o.params.as_deref().map(generics_of).unwrap_or_default());
        }
        if let Within::Impl(k) = f.within {
            gens.extend(impl_generics[k].iter().cloned());
        }
        let Some(mut cx) = cx_for(f.pos, gens) else { continue };
        cx.generic_bounds(&mut f.generics);
        if let Some(o) = f.owner.as_mut().and_then(|o| o.params.as_mut()) {
            cx.generic_bounds(o);
        }
        for p in f.params.iter_mut() {
            cx.local(&p.name, p.pos);
            cx.type_ref(&mut p.ty);
        }
        if let Some(r) = &mut f.ret {
            cx.type_ref(r);
        }
        // contract.decl.2: conditions name the module's items as the body does.
        for c in f.contracts.iter_mut() {
            cx.expr(&mut c.cond);
            for (_, e) in c.olds.iter_mut() {
                cx.expr(e);
            }
        }
        if let Some(b) = &mut f.body {
            cx.block(b);
        }
        diags.extend(cx.diags);
    }
    for imp in prog.impls.iter_mut() {
        if let Some(params) = imp.target.params.as_mut()
            && let Some(mut cx) = cx_for(imp.pos, params.iter().map(|g| g.name.clone()).collect())
        {
            cx.generic_bounds(params);
            diags.extend(cx.diags);
        }
    }
    crate::parser::link_inherited_contracts(prog);
}

/// mod.use.5: imports must not form a cycle. Each cycle is reported once,
/// at its first import in source order.
fn report_cycles(edges: &[(String, String, Pos)], diags: &mut Vec<Diagnostic>) {
    let mut sorted: Vec<&(String, String, Pos)> = edges.iter().collect();
    sorted.sort_by_key(|(_, _, p)| (p.line, p.col));
    let mut reported: HashSet<Vec<String>> = HashSet::default();
    for (from, to, pos) in &sorted {
        // A path back from `to` to `from` closes a cycle through this import.
        let Some(mut path) = find_path(edges, to, from) else { continue };
        path.insert(0, from.clone());
        let mut key = path[..path.len() - 1].to_vec();
        key.sort();
        if !reported.insert(key) {
            continue;
        }
        let shown: Vec<String> = path.iter().map(|m| if m.is_empty() { "(root)".into() } else { m.clone() }).collect();
        diags.push(
            Diagnostic::new("mod.use.5", *pos, format!("imports form a cycle: {}", shown.join(" -> ")))
                .help("move what the modules share into a module that both import"),
        );
    }
}

/// A path of imports from `from` to `to`, both included.
fn find_path(edges: &[(String, String, Pos)], from: &str, to: &str) -> Option<Vec<String>> {
    let mut prev: HashMap<&str, &str> = HashMap::default();
    let mut todo = vec![from];
    let mut seen: HashSet<&str> = HashSet::from_iter([from]);
    while let Some(m) = todo.pop() {
        if m == to {
            let mut path = vec![to.to_string()];
            let mut at = to;
            while at != from {
                at = prev[at];
                path.push(at.to_string());
            }
            path.reverse();
            return Some(path);
        }
        let mut next: Vec<&str> = edges.iter().filter(|(a, _, _)| a == m).map(|(_, b, _)| b.as_str()).collect();
        next.sort();
        for n in next.into_iter().rev() {
            if seen.insert(n) {
                prev.insert(n, m);
                todo.push(n);
            }
        }
    }
    None
}

/// Name resolution inside one declaration of one file.
struct Cx<'a> {
    module: String,
    items: &'a Items,
    imports: &'a HashMap<String, Target>,
    public: &'a HashSet<usize>,
    /// Type parameters in scope, which no item renaming touches.
    generics: Vec<String>,
    diags: Vec<Diagnostic>,
}

impl Cx<'_> {
    fn own(&self, name: &str) -> bool {
        !self.generics.iter().any(|g| g == name) && self.items.get(&self.module).is_some_and(|m| m.contains_key(name))
    }

    fn import(&self, name: &str) -> Option<&Target> {
        self.imports.get(name)
    }

    /// The qualified name of item `item` of imported module `m`, reporting
    /// an unknown or private item (mod.use.2, mod.vis.1).
    fn item(&mut self, m: &str, alias: &str, item: &str, pos: Pos) -> String {
        match self.items.get(m).and_then(|names| names.get(item)) {
            None => {
                let mut d = Diagnostic::new("mod.use.2", pos, format!("module `{m}` has no item `{item}`"));
                let mut names: Vec<&str> =
                    self.items.get(m).map(|n| n.keys().map(String::as_str).collect()).unwrap_or_default();
                names.sort();
                if let Some(s) = crate::check::suggest(item, &names) {
                    d = d.help(format!("did you mean `{alias}.{s}`?"));
                } else if names.is_empty() {
                    d = d.help(format!("module `{m}` has no public items"));
                } else {
                    let list: Vec<String> = names.iter().take(12).map(|n| format!("`{alias}.{n}`")).collect();
                    d = d.help(format!("the module's items include {}", list.join(", ")));
                }
                self.diags.push(d);
            }
            Some(decl) if !is_prelude(*decl) && !self.public.contains(&decl.line) => {
                self.diags.push(
                    Diagnostic::new("mod.vis.1", pos, format!("`{item}` is private to module `{m}`"))
                        .help(format!("mark it `pub` in module `{m}` to use it here")),
                );
            }
            Some(_) => {}
        }
        qualified(m, item)
    }

    /// A type or trait name as written: `T`, or `m.T` through an import.
    fn type_name(&mut self, name: &str, pos: Pos) -> String {
        let Some((head, rest)) = name.split_once('.') else {
            return if self.own(name) { qualified(&self.module, name) } else { name.to_string() };
        };
        match self.import(head).cloned() {
            Some(Target::Module(m)) => self.item(&m, head, rest, pos),
            Some(Target::Std) => name.to_string(),
            // A standard module used without its import: std.use.2, reported
            // by the checker.
            None if crate::stdlib::is_std_module(head) => name.to_string(),
            None => {
                self.diags.push(
                    Diagnostic::new("mod.use.2", pos, format!("`{head}` is not an imported module"))
                        .help(format!("add `use {head}` at the top of this file")),
                );
                name.to_string()
            }
        }
    }

    fn type_ref(&mut self, t: &mut TypeRef) {
        match &mut t.kind {
            TypeKind::Named { name, args } => {
                *name = self.type_name(name, t.pos);
                args.iter_mut().for_each(|a| self.type_ref(a));
            }
            TypeKind::Optional(inner) | TypeKind::Ref(inner) | TypeKind::Ptr(inner) => self.type_ref(inner),
            TypeKind::Fn { params, ret } => {
                params.iter_mut().for_each(|p| self.type_ref(p));
                if let Some(r) = ret {
                    self.type_ref(r);
                }
            }
            TypeKind::Fallible { ok, errors } => {
                if let Some(ok) = ok {
                    self.type_ref(ok);
                }
                errors.iter_mut().flatten().for_each(|e| self.type_ref(e));
            }
            TypeKind::Any(name) => *name = self.type_name(name, t.pos),
        }
    }

    fn generic_bounds(&mut self, gs: &mut [GenericParam]) {
        for (b, pos) in gs.iter_mut().flat_map(|g| g.bounds.iter_mut()) {
            *b = self.type_name(b, *pos);
        }
    }

    /// expr.bind.5: a local can't take the name of an item of its module or
    /// of an import. The checker reports clashes with root-module items.
    fn local(&mut self, name: &str, pos: Pos) {
        // A standard module's name is the checker's (expr.bind.5).
        let what = if matches!(self.import(name), Some(Target::Module(_))) {
            "an imported module"
        } else if !self.module.is_empty() && self.own(name) {
            "an item"
        } else {
            return;
        };
        self.diags.push(
            Diagnostic::new("expr.bind.5", pos, format!("`{name}` is already {what} name"))
                .help("choose a different name for the local"),
        );
    }

    fn block(&mut self, b: &mut Block) {
        for s in b {
            self.stmt(s);
        }
    }

    fn stmt(&mut self, s: &mut Stmt) {
        match s {
            Stmt::Let { name, pos, value, otherwise, .. } => {
                self.expr(value);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
                self.local(name, *pos);
            }
            Stmt::Assign { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::For { var, var_pos, iter, body, .. } | Stmt::With { mutex: iter, var, var_pos, body, .. } => {
                self.expr(iter);
                self.local(var, *var_pos);
                self.block(body);
            }
            Stmt::Unsafe { body, .. } => self.block(body),
            Stmt::Return(Some(e), _) | Stmt::Expr(e) => self.expr(e),
            Stmt::Handle { value, otherwise } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Stmt::Defer(inner, _) => self.stmt(inner),
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
        }
    }

    fn otherwise(&mut self, o: &mut Else) {
        match o {
            Else::Value(e) => self.expr(e),
            Else::Leave(s) => self.stmt(s),
            Else::Block { err, body, .. } => {
                if let Some((name, pos)) = err {
                    self.local(name, *pos);
                }
                self.block(body);
            }
        }
    }

    fn pattern(&mut self, p: &mut Pattern) {
        match p {
            Pattern::Variant { qualifier, bindings, pos, .. } => {
                if let Some(q) = qualifier {
                    *q = self.type_name(q, *pos);
                }
                for (name, pos) in bindings.iter().flatten() {
                    self.local(name, *pos);
                }
            }
            Pattern::Or(alts, _) => alts.iter_mut().for_each(|a| self.pattern(a)),
            Pattern::Wildcard(_) | Pattern::Literal(..) | Pattern::Range { .. } => {}
        }
    }

    /// The imported module that `e` names, if it's a bare import name.
    fn module_name(&self, e: &Expr) -> Option<(String, String, Pos)> {
        match e {
            Expr::Name(n, p) => match self.import(n) {
                Some(Target::Module(m)) => Some((m.clone(), n.clone(), *p)),
                _ => None,
            },
            _ => None,
        }
    }

    fn expr(&mut self, e: &mut Expr) {
        match e {
            Expr::Name(n, pos) => {
                if self.import(n).is_some_and(|t| matches!(t, Target::Module(_))) {
                    self.diags.push(
                        Diagnostic::new("mod.use.2", *pos, format!("`{n}` is a module, not a value"))
                            .help(format!("use one of its items, e.g. `{n}.name`")),
                    );
                } else if self.own(n) && !self.module.is_empty() {
                    *n = qualified(&self.module, n);
                }
            }
            Expr::TypeArgs { name, args, pos } => {
                *name = self.type_name(name, *pos);
                args.iter_mut().for_each(|a| self.type_ref(a));
            }
            Expr::Field { recv, name, name_pos } => {
                if let Some((m, alias, pos)) = self.module_name(recv) {
                    let q = self.item(&m, &alias, name, *name_pos);
                    *e = Expr::Name(q, pos);
                } else {
                    self.expr(recv);
                }
            }
            Expr::Method { recv, name, name_pos, args } => {
                args.iter_mut().for_each(|a| self.expr(&mut a.value));
                if let Some((m, alias, pos)) = self.module_name(recv) {
                    let q = self.item(&m, &alias, name, *name_pos);
                    let args = std::mem::take(args);
                    *e = Expr::Call { callee: Box::new(Expr::Name(q, pos)), args, pos: *name_pos };
                } else {
                    self.expr(recv);
                }
            }
            Expr::Call { callee, args, .. } => {
                self.expr(callee);
                args.iter_mut().for_each(|a| self.expr(&mut a.value));
            }
            Expr::Interp(parts, _) => {
                for p in parts {
                    if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                        self.expr(x);
                    }
                }
            }
            Expr::List(items, _) | Expr::Set(items, _) => items.iter_mut().for_each(|x| self.expr(x)),
            Expr::Map(pairs, _) => pairs.iter_mut().for_each(|(k, v)| {
                self.expr(k);
                self.expr(v);
            }),
            Expr::Comp { elem, value, clauses, .. } => {
                for c in clauses.iter_mut() {
                    self.expr(&mut c.iter);
                    self.local(&c.var, c.var_pos);
                    if let Some(cond) = &mut c.cond {
                        self.expr(cond);
                    }
                }
                self.expr(elem);
                if let Some(v) = value {
                    self.expr(v);
                }
            }
            Expr::Index { recv, index, .. } => {
                self.expr(recv);
                self.expr(index);
            }
            Expr::Unary { operand, .. } | Expr::Ref { value: operand, .. } => self.expr(operand),
            Expr::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Range { lo, hi, .. } => {
                if let Some(x) = lo {
                    self.expr(x);
                }
                if let Some(x) = hi {
                    self.expr(x);
                }
            }
            Expr::If { branches, els, else_err, .. } => {
                for (cond, body) in branches.iter_mut() {
                    match cond {
                        Cond::Expr(c) => self.expr(c),
                        Cond::Bind { name, pos, value } => {
                            self.expr(value);
                            self.local(name, *pos);
                        }
                    }
                    self.block(body);
                }
                if let Some((name, pos)) = else_err {
                    self.local(name, *pos);
                }
                if let Some(b) = els {
                    self.block(b);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                for arm in arms.iter_mut() {
                    self.pattern(&mut arm.pat);
                    if let Some(g) = &mut arm.guard {
                        self.expr(g);
                    }
                    self.block(&mut arm.body);
                }
            }
            Expr::Else { value, otherwise, .. } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Expr::Try { expr, context, .. } => {
                self.expr(expr);
                if let Some(c) = context {
                    self.expr(c);
                }
            }
            Expr::Str(..)
            | Expr::Bytes(..)
            | Expr::Int(..)
            | Expr::Float(..)
            | Expr::Char(..)
            | Expr::Bool(..)
            | Expr::None(_)
            | Expr::Hole(_) => {}
        }
    }
}

/// The file path and in-file position of `pos`. Prelude positions stay as
/// they are, reported against the first file.
pub fn locate(files: &[SourceFile], pos: Pos) -> (&str, Pos) {
    let first = files.first().map_or("", |f| f.path.as_str());
    if is_prelude(pos) {
        return (first, pos);
    }
    match files.iter().rev().find(|f| f.base < pos.line) {
        Some(f) => (f.path.as_str(), Pos { line: pos.line - f.base, col: pos.col }),
        None => (first, pos),
    }
}

/// Report diagnostics file by file, each against its own source. A note
/// in another file names that file.
pub fn report(files: &[SourceFile], diags: Vec<Diagnostic>, json: bool) {
    let index = |pos: Pos| if is_prelude(pos) { 0 } else { files.iter().rposition(|f| f.base < pos.line).unwrap_or(0) };
    let local = |pos: Pos| locate(files, pos).1;
    let mut groups: Vec<Vec<Diagnostic>> = files.iter().map(|_| Vec::new()).collect();
    for mut d in diags {
        let fi = index(d.pos);
        d.pos = local(d.pos);
        d.end = d.end.map(local);
        for n in d.notes.iter_mut() {
            // ct.gen.3: a note in generated code already gives its address.
            if index(n.pos) != fi && !files[index(n.pos)].path.starts_with("gen:") {
                n.message = format!("{} (in {})", n.message, files[index(n.pos)].path);
            }
            n.pos = local(n.pos);
        }
        for e in d.fixes.iter_mut().flat_map(|f| f.edits.iter_mut()) {
            e.pos = local(e.pos);
            e.end = local(e.end);
        }
        groups[fi].push(d);
    }
    for (f, mut g) in files.iter().zip(groups) {
        if !g.is_empty() {
            crate::diag::report(&mut g, &f.path, &f.src, json);
        }
    }
}
