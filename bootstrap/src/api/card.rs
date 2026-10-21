//! `ail api --card [--budget N] [--json] DIR|FILE`: a program's own card,
//! the whole program in one read under a token budget (plan §2.3, ADR 0310).
//!
//! Spec rules: api.card.1-5. The card is the outline of `ail api` per
//! module, plus the entry point, the tests, the manifest and the examples.
//! Over budget it drops, in this order: private items, doc comments,
//! parameter names, member lists, then everything but module names
//! (api.card.3), and its last line says what it dropped.

use std::fmt::Write as _;
use std::path::Path;
use std::process::ExitCode;

use super::{API_VERSION, Api, Item, Json};
use crate::ast::*;
use crate::check::{self, Resolved};
use crate::diag;
use crate::docs;
use crate::modules;

pub const USAGE: &str = "usage: ail api --card [--budget N] [--json] [--module NAME | --index] DIR|FILE\n";

/// The default budget in estimated tokens (api.card.3, ADR 0311).
pub const DEFAULT_BUDGET: usize = 4000;

/// How the root module (named "" internally) is written on the command line
/// and in cards.
const ROOT_TITLE: &str = "(root)";

/// How much of the card is left, from everything (0) to module names (6).
const LEVELS: usize = 6;

/// What each level drops, for the card's last line (api.card.3).
const DROPPED: [&str; LEVELS] = [
    "private items",
    "doc comments",
    "parameter names",
    "members, contracts and examples (docs past their first sentence)",
    "fields, variants and test names",
    "signatures (names only)",
];

/// `ail api --card ...`.
pub fn command(args: &[&str], json_diags: bool) -> ExitCode {
    let mut budget = DEFAULT_BUDGET;
    // The global `--json` flag reaches here as `json_diags`.
    let mut json = json_diags;
    let mut path = None;
    let mut module = None;
    let mut index = false;
    let mut it = args.iter().copied().filter(|a| *a != "--card");
    while let Some(a) = it.next() {
        match a {
            "--json" => json = true,
            "--index" => index = true,
            "--module" => match it.next() {
                Some(m) => module = Some(m),
                None => {
                    eprint!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            "--budget" => match it.next().and_then(|n| n.parse::<usize>().ok()) {
                Some(n) if n > 0 => budget = n,
                _ => {
                    eprint!("{USAGE}");
                    return ExitCode::from(2);
                }
            },
            f if !f.starts_with("--") && path.is_none() => path = Some(f),
            _ => {
                eprint!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(path) = path else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    if index && module.is_some() {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    }
    let out = if index {
        self::index(path, json)
    } else if let Some(m) = module {
        module_card(path, m, budget, json)
    } else {
        card(path, budget, json)
    };
    match out {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(Failure::Message(m)) => {
            eprintln!("error: {m}");
            ExitCode::from(2)
        }
        Err(Failure::Diags(files, diags)) => {
            modules::report(&files, diags, json_diags);
            ExitCode::from(2)
        }
    }
}

pub enum Failure {
    Message(String),
    Diags(Vec<SourceFile>, Vec<diag::Diagnostic>),
}

/// One declaration of the card.
struct Entry {
    public: bool,
    /// `struct`, `enum`, `fn`, `method`, `field`, ...
    kind: &'static str,
    name: String,
    /// The signature as `ail api` writes it, parameter names included.
    signature: String,
    /// The same without parameter names; `None` unless it has parameters.
    bare: Option<String>,
    doc: Option<String>,
    requires: Vec<String>,
    ensures: Vec<String>,
    fails: Vec<String>,
    effects: Vec<String>,
    members: Vec<Entry>,
    /// Where it is declared: the file as loaded and the line and column in
    /// it (api.card.2); only the JSON form shows it.
    file: String,
    line: usize,
    col: usize,
}

struct Module {
    name: String,
    entries: Vec<Entry>,
}

struct Card {
    name: String,
    entry: Option<String>,
    tests: Vec<String>,
    /// The module of each test, parallel to `tests` (api.card.6).
    test_modules: Vec<String>,
    manifest: Vec<String>,
    examples: Vec<(String, String)>,
    modules: Vec<Module>,
}

/// The card of `path` (a package directory or a file) within `budget`
/// estimated tokens, as Markdown or JSON (api.card.1-2).
pub fn card(path: &str, budget: usize, json: bool) -> Result<String, Failure> {
    fit(build(path)?, budget, json)
}

/// `ail api --card --module NAME`: the card of one module of the package
/// (api.card.6). The package's entry, manifest and examples are the root
/// module's, so they appear only on the root module's card.
pub fn module_card(path: &str, module: &str, budget: usize, json: bool) -> Result<String, Failure> {
    let mut built = build(path)?;
    let name = if module == ROOT_TITLE { "" } else { module };
    if !built.modules.iter().any(|m| m.name == name) {
        return Err(Failure::Message(format!(
            "no module `{module}` in {path}; `ail api --card --index {path}` lists the modules"
        )));
    }
    built.modules.retain(|m| m.name == name);
    built.name = format!("{}: {}", built.name, module_title_of(name));
    if !name.is_empty() {
        built.entry = None;
        built.manifest.clear();
        built.examples.clear();
    }
    let (tests, test_modules): (Vec<String>, Vec<String>) =
        built.tests.drain(..).zip(built.test_modules.drain(..)).filter(|(_, m)| m.as_str() == name).unzip();
    built.tests = tests;
    built.test_modules = test_modules;
    fit(built, budget, json)
}

/// `ail api --card --index`: one line per module with its item count, so an
/// agent reads the index and then one module card (api.card.7).
pub fn index(path: &str, json: bool) -> Result<String, Failure> {
    let built = build(path)?;
    if json {
        let modules = built
            .modules
            .iter()
            .map(|m| {
                Json::Obj(vec![("module", Json::Str(m.name.clone())), ("count", Json::Int(m.entries.len() as i64))])
            })
            .collect();
        let top = vec![
            ("api", Json::Int(API_VERSION)),
            ("card", Json::Str(built.name.clone())),
            ("modules", Json::Arr(modules)),
        ];
        let mut out = String::new();
        Json::Obj(top).write(&mut out, None);
        out.push('\n');
        return Ok(out);
    }
    let mut out = String::new();
    writeln!(out, "# {}", built.name).unwrap();
    for m in &built.modules {
        writeln!(out, "{}  {} items", module_title(m), m.entries.len()).unwrap();
    }
    Ok(out)
}

/// The richest card of `built` within `budget` (api.card.3).
fn fit(built: Card, budget: usize, json: bool) -> Result<String, Failure> {
    // api.card.3 (ADR 0703): the richest level within the budget. The levels
    // are not nested (the first-sentence level keeps docs that a level after
    // it drops), so the first level that fits is not always the fullest one.
    // "Richest" counts the card's content, not its trailer (ADR 0719): a
    // deeper level's trailer lists more dropped items, so counting it would
    // let a trimmed card outrank a fuller one.
    let mut best: Option<(usize, usize)> = None;
    for level in 0..=LEVELS {
        let content = estimate_tokens(&render(&built, level, json, budget, false));
        let full = estimate_tokens(&render(&built, level, json, budget, true));
        if full <= budget && best.is_none_or(|(t, _)| content > t) {
            best = Some((content, level));
        }
    }
    let level = best.map_or(LEVELS, |(_, level)| level);
    Ok(render(&built, level, json, budget, true))
}

fn build(path: &str) -> Result<Card, Failure> {
    let p = Path::new(path);
    let root = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf()
    };
    let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string());
    let (mut prog, mut diags) = modules::load(path).map_err(Failure::Message)?;
    let mut resolved = Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&prog);
    }
    // A library has no `main` (api.out.5).
    diags.retain(|d| d.rule != "mod.main.1");
    if diags.iter().any(|d| !d.warning) {
        return Err(Failure::Diags(prog.files, diags));
    }
    // Doc comments, file by file: a position's line is its file's base plus
    // the line in the file (api.card.1).
    let mut by_line = std::collections::HashMap::new();
    for f in prog.files.iter().filter(|f| f.path.ends_with(".ail")) {
        for (line, doc) in docs::by_line(&f.src) {
            by_line.insert(f.base + line, doc);
        }
    }
    docs::attach_lines(&mut prog, &by_line);
    let effects = check::effects::effects_of(&prog, &resolved);
    let mut api = Api::new(&prog, effects);
    api.errors = std::mem::take(&mut resolved.fn_errors);
    api.whole = true;

    let tests: Vec<usize> =
        prog.fns.iter().enumerate().filter(|(_, f)| f.has_attr("test") && !is_prelude(f.pos)).map(|(i, _)| i).collect();
    let test_lines: std::collections::HashSet<usize> = tests.iter().map(|&i| prog.fns[i].pos.line).collect();
    let mut modules: Vec<Module> = Vec::new();
    for item in api.outline_items() {
        if test_lines.contains(&item.pos.line) {
            continue;
        }
        let module = prog.module_at(item.pos).to_string();
        let entry = entry_of(&prog, item);
        match modules.iter_mut().find(|m| m.name == module) {
            Some(m) => m.entries.push(entry),
            None => modules.push(Module { name: module, entries: vec![entry] }),
        }
    }
    // Deterministic: the root module first, then by import path (api.card.1).
    modules.sort_by(|a, b| (!a.name.is_empty(), &a.name).cmp(&(!b.name.is_empty(), &b.name)));

    let entry = prog
        .fns
        .iter()
        .find(|f| f.name == "main" && f.owner.is_none() && prog.module_at(f.pos).is_empty() && !is_prelude(f.pos))
        .map(|f| {
            let file = prog.file_at(f.pos).map(|sf| file_label(&root, &sf.path)).unwrap_or_default();
            format!("main in {file}")
        });
    let mut tests_named: Vec<(String, String)> = tests
        .iter()
        .map(|&i| {
            let f = &prog.fns[i];
            let m = prog.module_at(f.pos);
            let qualified = if m.is_empty() { f.name.clone() } else { format!("{m}.{}", f.name) };
            (m.to_string(), qualified)
        })
        .collect();
    tests_named.sort_by(|a, b| a.1.cmp(&b.1));
    let (test_modules, test_names): (Vec<String>, Vec<String>) = tests_named.into_iter().unzip();

    let manifest = std::fs::read_to_string(root.join(crate::packages::MANIFEST))
        .map(|s| s.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).map(String::from).collect())
        .unwrap_or_default();
    Ok(Card { name, entry, tests: test_names, test_modules, manifest, examples: examples(&root), modules })
}

/// `edit.ail`, or `geo/shapes.ail`: the file's path below the package root.
fn file_label(root: &Path, file: &str) -> String {
    Path::new(file).strip_prefix(root).map_or_else(|_| file.to_string(), |p| p.to_string_lossy().replace('\\', "/"))
}

/// The `.ail` files of `root/examples` with the first line of each file's
/// leading doc comment (api.card.1), sorted by name.
fn examples(root: &Path) -> Vec<(String, String)> {
    let Ok(dir) = std::fs::read_dir(root.join("examples")) else { return Vec::new() };
    let mut out: Vec<(String, String)> = dir
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "ail"))
        .map(|e| {
            let src = std::fs::read_to_string(e.path()).unwrap_or_default();
            let first = src
                .lines()
                .find_map(|l| l.trim_start().strip_prefix("##"))
                .map(|d| d.trim().to_string())
                .unwrap_or_default();
            (e.file_name().to_string_lossy().into_owned(), first)
        })
        .collect();
    out.sort();
    out
}

fn entry_of(prog: &Program, item: Item) -> Entry {
    let strings = |key: &str| -> Vec<String> {
        match item.extra.iter().find(|(k, _)| *k == key) {
            Some((_, Json::Arr(xs))) => {
                xs.iter().filter_map(|x| if let Json::Str(s) = x { Some(s.clone()) } else { None }).collect()
            }
            _ => Vec::new(),
        }
    };
    let bare = match item.extra.iter().find(|(k, _)| *k == "params") {
        Some((_, Json::Arr(ps))) if !ps.is_empty() => Some(bare_signature(&item.signature, ps)),
        _ => None,
    };
    let (requires, ensures, fails, effects) =
        (strings("requires"), strings("ensures"), strings("fails_with"), strings("effects"));
    let public = prog.is_pub(item.pos);
    let (file, at) = crate::modules::locate(&prog.files, item.pos);
    let file = file.to_string();
    let members = item.members.into_iter().flat_map(|(_, ms)| ms).map(|m| entry_of(prog, m)).collect();
    Entry {
        file,
        line: at.line,
        col: at.col,
        public,
        kind: item.kind,
        name: item.name,
        signature: item.signature,
        bare,
        doc: item.doc,
        requires,
        ensures,
        fails,
        effects,
        members,
    }
}

/// `fn name(int, str) T`: the signature with each parameter's name removed
/// (`mut` and `move` kept). The parameter list is the first balanced `(...)`.
fn bare_signature(sig: &str, params: &[Json]) -> String {
    let Some(open) = sig.find('(') else { return sig.to_string() };
    let mut depth = 0;
    let mut close = sig.len();
    for (i, c) in sig.char_indices().skip(open) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = i;
                    break;
                }
            }
            _ => {}
        }
    }
    let mut parts: Vec<String> = Vec::new();
    // A receiver is written first and isn't among `params`.
    let inner = &sig[open + 1..close.min(sig.len())];
    if inner.starts_with("mut self") {
        parts.push("mut self".into());
    } else if inner.starts_with("self") {
        parts.push("self".into());
    }
    for p in params {
        if let Json::Obj(kvs) = p {
            let get = |k: &str| kvs.iter().find(|(n, _)| *n == k).map(|(_, v)| v);
            let ty = if let Some(Json::Str(t)) = get("type") { t.clone() } else { String::new() };
            let mode = match (get("mut"), get("move")) {
                (Some(Json::Bool(true)), _) => "mut ",
                (_, Some(Json::Bool(true))) => "move ",
                _ => "",
            };
            parts.push(format!("{mode}{ty}"));
        }
    }
    format!("{}({}){}", &sig[..open], parts.join(", "), &sig[(close + 1).min(sig.len())..])
}

/// Whether `e` is dropped at `level` for being private: not `pub`, in a
/// module that exports something or isn't the root (api.card.3). A root
/// module without any `pub` has no importers, so all of it is its surface.
fn private(module: &Module, e: &Entry) -> bool {
    !e.public && (!module.name.is_empty() || module.entries.iter().any(|x| x.public))
}

/// The card at `level`; `trailer` says whether it ends with what it dropped
/// (and the budget), which `fit` leaves out when it ranks the levels.
fn render(card: &Card, level: usize, json: bool, budget: usize, trailer: bool) -> String {
    let dropped: Vec<&str> = if trailer { DROPPED[..level].to_vec() } else { Vec::new() };
    if json {
        render_json(card, level, &dropped, budget, trailer)
    } else {
        render_md(card, level, &dropped, budget, trailer)
    }
}

fn shown<'a>(module: &'a Module, level: usize) -> Vec<&'a Entry> {
    module.entries.iter().filter(|e| level == 0 || !private(module, e)).collect()
}

fn signature<'a>(e: &'a Entry, level: usize) -> &'a str {
    if level >= 3 { e.bare.as_deref().unwrap_or(&e.signature) } else { &e.signature }
}

fn first_line(doc: &str) -> &str {
    doc.lines().next().unwrap_or("").trim()
}

/// The first sentence of a doc comment: its first line, cut after the first
/// `. ` (api.card.3, ADR 0703).
fn first_sentence(doc: &str) -> &str {
    let line = first_line(doc);
    match line.find(". ") {
        Some(i) => &line[..=i],
        None => line,
    }
}

fn line(e: &Entry, level: usize, indent: &str, out: &mut String) {
    let vis = if e.public { "pub " } else { "" };
    if level >= 6 {
        writeln!(out, "{indent}{} {}", e.kind, e.name).unwrap();
        return;
    }
    write!(out, "{indent}{vis}{}", signature(e, level)).unwrap();
    if !e.effects.is_empty() {
        write!(out, " [{}]", e.effects.join(" ")).unwrap();
    }
    if level < 2
        && let Some(d) = &e.doc
    {
        write!(out, "  ## {}", first_line(d)).unwrap();
    } else if level == 4
        && let Some(d) = &e.doc
    {
        write!(out, "  ## {}", first_sentence(d)).unwrap();
    }
    out.push('\n');
    if level < 4 {
        for c in &e.requires {
            writeln!(out, "{indent}    requires {c}").unwrap();
        }
        for c in &e.ensures {
            writeln!(out, "{indent}    ensures {c}").unwrap();
        }
        if !e.fails.is_empty() && e.signature.contains(") !") {
            writeln!(out, "{indent}    fails with {}", e.fails.join(" | ")).unwrap();
        }
    }
}

fn render_md(card: &Card, level: usize, dropped: &[&str], budget: usize, trailer: bool) -> String {
    let mut out = String::new();
    writeln!(out, "# {}", card.name).unwrap();
    if let Some(e) = &card.entry {
        writeln!(out, "entry: {e}").unwrap();
    }
    if level >= 4 && !card.tests.is_empty() {
        writeln!(out, "tests: {} (`ail test` lists them)", card.tests.len()).unwrap();
    } else if !card.tests.is_empty() {
        writeln!(out, "tests: {}", card.tests.join(", ")).unwrap();
    }
    if !card.manifest.is_empty() {
        writeln!(out, "ail.pkg:").unwrap();
        for l in &card.manifest {
            writeln!(out, "    {l}").unwrap();
        }
    }
    if !card.examples.is_empty() && level < 4 {
        writeln!(out, "examples:").unwrap();
        for (f, d) in &card.examples {
            if d.is_empty() || level >= 2 { writeln!(out, "    {f}") } else { writeln!(out, "    {f}  ## {d}") }
                .unwrap();
        }
    }
    for m in &card.modules {
        let es = shown(m, level);
        if level >= 6 {
            writeln!(out, "\n## {} ({} items)", module_title(m), m.entries.len()).unwrap();
            continue;
        }
        if es.is_empty() {
            continue;
        }
        writeln!(out, "\n## {}", module_title(m)).unwrap();
        out.push_str("```\n");
        for e in es {
            line(e, level, "", &mut out);
            if level < 4 {
                for mem in &e.members {
                    if level == 0 || e.kind == "trait" || e.kind == "impl" || mem.kind == "method" || mem.kind == "fn" {
                        if level > 0 && private_member(mem) {
                            continue;
                        }
                        line(mem, level, "    ", &mut out);
                    } else {
                        line(mem, level, "    ", &mut out);
                    }
                }
            }
        }
        out.push_str("```\n");
    }
    if level > 0 && trailer {
        let est = estimate_tokens(&out);
        writeln!(out, "\ntrimmed to fit {budget} tokens (~{est}); dropped: {}", dropped.join(", ")).unwrap();
    }
    out
}

/// A method of a type that isn't `pub` (a field or variant has no
/// visibility of its own here).
fn private_member(m: &Entry) -> bool {
    matches!(m.kind, "method" | "fn") && !m.public
}

fn module_title(m: &Module) -> String {
    module_title_of(&m.name)
}

fn module_title_of(name: &str) -> String {
    if name.is_empty() { ROOT_TITLE.to_string() } else { name.to_string() }
}

fn entry_json(e: &Entry, level: usize) -> Json {
    let mut kvs = vec![
        ("kind", Json::Str(e.kind.into())),
        ("name", Json::Str(e.name.clone())),
        ("pub", Json::Bool(e.public)),
        // api.card.2: the declaration's position, so a tool can jump to it.
        ("file", Json::Str(e.file.clone())),
        ("line", Json::Int(e.line as i64)),
        ("col", Json::Int(e.col as i64)),
    ];
    if level < 6 {
        kvs.push(("signature", Json::Str(signature(e, level).to_string())));
        if level < 2 {
            kvs.push(("doc", Json::opt(e.doc.as_deref().map(|d| first_line(d).to_string()))));
        } else if level == 4 {
            kvs.push(("doc", Json::opt(e.doc.as_deref().map(|d| first_sentence(d).to_string()))));
        }
        if !e.effects.is_empty() {
            kvs.push(("effects", Json::Arr(e.effects.iter().cloned().map(Json::Str).collect())));
        }
        if level < 4 {
            for (k, v) in [("requires", &e.requires), ("ensures", &e.ensures), ("fails_with", &e.fails)] {
                if !v.is_empty() {
                    kvs.push((k, Json::Arr(v.iter().cloned().map(Json::Str).collect())));
                }
            }
            let members: Vec<Json> =
                e.members.iter().filter(|m| level == 0 || !private_member(m)).map(|m| entry_json(m, level)).collect();
            if !members.is_empty() {
                kvs.push(("members", Json::Arr(members)));
            }
        }
    }
    Json::Obj(kvs)
}

fn render_json(card: &Card, level: usize, dropped: &[&str], budget: usize, trailer: bool) -> String {
    let strs = |v: &[String]| Json::Arr(v.iter().cloned().map(Json::Str).collect());
    let modules = card
        .modules
        .iter()
        .map(|m| {
            let items = if level >= 6 {
                Vec::new()
            } else {
                shown(m, level).into_iter().map(|e| entry_json(e, level)).collect()
            };
            Json::Obj(vec![
                ("module", Json::Str(m.name.clone())),
                ("count", Json::Int(m.entries.len() as i64)),
                ("items", Json::Arr(items)),
            ])
        })
        .collect();
    let examples = card
        .examples
        .iter()
        .filter(|_| level < 4)
        .map(|(f, d)| {
            Json::Obj(vec![
                ("file", Json::Str(f.clone())),
                ("doc", Json::Str(if level >= 2 { String::new() } else { d.clone() })),
            ])
        })
        .collect();
    let top = vec![
        ("api", Json::Int(API_VERSION)),
        ("card", Json::Str(card.name.clone())),
        ("entry", Json::opt(card.entry.clone())),
        ("tests", if level >= 4 { Json::Arr(Vec::new()) } else { strs(&card.tests) }),
        ("test_count", Json::Int(card.tests.len() as i64)),
        ("manifest", strs(&card.manifest)),
        ("examples", Json::Arr(examples)),
        ("modules", Json::Arr(modules)),
    ];
    let top = if trailer {
        top.into_iter()
            .chain([
                ("budget", Json::Int(budget as i64)),
                ("dropped", Json::Arr(dropped.iter().map(|d| Json::Str(d.to_string())).collect())),
            ])
            .collect()
    } else {
        top
    };
    let mut out = String::new();
    Json::Obj(top).write(&mut out, None);
    out.push('\n');
    out
}

/// A cheap token estimate (api.card.3): a run of letters or digits costs one
/// token per four characters (at least one), any other non-space character
/// one token, a newline one. Against `tools/tokbench` it lands within 3% to
/// 17% above the largest of the five tokenizers on cards over 300 tokens, so
/// a card within budget here is within budget for each of them (ADR 0311).
pub fn estimate_tokens(s: &str) -> usize {
    let mut tokens: usize = 0;
    let mut run: usize = 0;
    for c in s.chars() {
        if c.is_alphanumeric() || c == '_' {
            run += 1;
            continue;
        }
        if run > 0 {
            tokens += run.div_ceil(5);
            run = 0;
        }
        if !c.is_whitespace() || c == '\n' {
            tokens += 1;
        }
    }
    tokens + run.div_ceil(5)
}
