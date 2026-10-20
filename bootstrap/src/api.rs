//! `ail api FILE [--text] [--symbols]`: a program's outline without its
//! bodies, for AI agents and tools (plan §2.3-2.4, ADR 0150, ADR 0151).
//!
//! Spec rules: api.out.1-5 (the JSON shape), api.id.1-3 (stable symbol ids),
//! api.ref.1-2 (references), api.text.1 (the text outline). See
//! `docs/design/semantic-api.md`.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::process::ExitCode;

use crate::ast::*;
use crate::check::{self, Call, Resolved};
use crate::diag::{self, Pos};
use crate::docs;
use crate::modules;

pub mod card;
pub mod errors;
mod expanded;
mod query;
pub mod syntax;

/// Whether `pos` is outside the file being outlined: in the prelude or in
/// an imported module (`modules.rs`).
fn is_prelude(pos: Pos) -> bool {
    pos.line > FILE_LINES
}

/// The version of the output shape (api.out.1). Bump it when a key changes meaning.
pub const API_VERSION: i64 = 1;

pub const USAGE: &str = "usage: ail api --card [--budget N] [--json] DIR|FILE\n       ail api (--tokens | --ast) FILE\n       ail api --ast --program DIR|FILE\n       ail api [--outline] DIR|FILE [--text | --pretty] [--symbols] [--expanded] [--json]\n       ail api (--locals | --type | --refs) [DIR] FILE LINE COL [--pretty] [--json]\n";

/// How the outline is written (api.out.2, api.text.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// JSON on one line, the default: the fewest tokens for a reader.
    Json,
    /// JSON with a two-space indent, for a human (`--pretty`).
    Pretty,
    /// The compact outline (`--text`).
    Text,
}

/// `ail api ...`; `json` asks for diagnostics as JSON lines.
pub fn command(args: &[&str], json: bool) -> ExitCode {
    if args.contains(&"--card") {
        return card::command(args, json);
    }
    // api.tok.1, api.ast.1: the lexer's tokens and the parser's tree of one file.
    if args.contains(&"--tokens") || args.contains(&"--ast") {
        return syntax::command(args);
    }
    if let Some(q) = args.iter().find_map(|a| query::Query::from_flag(a)) {
        return query::command(q, args, json);
    }
    let mut form = Form::Json;
    let mut symbols = false;
    let mut expanded = false;
    let mut file = None;
    for a in args {
        match *a {
            "--text" if form == Form::Json => form = Form::Text,
            "--pretty" if form == Form::Json => form = Form::Pretty,
            "--symbols" => symbols = true,
            "--outline" => {}
            "--expanded" => expanded = true,
            f if !f.starts_with("--") && file.is_none() => file = Some(f),
            _ => {
                eprint!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(path) = file else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    // api.pkg.1: a package directory is outlined whole, with module-prefixed ids.
    if std::path::Path::new(path).is_dir() {
        return match package_outline(path, symbols, form) {
            Ok(out) => {
                print!("{out}");
                ExitCode::SUCCESS
            }
            Err(PackageFailure::Message(m)) => {
                eprintln!("error: {m}");
                ExitCode::from(2)
            }
            Err(PackageFailure::Diags(files, diags)) => {
                modules::report(&files, diags, json);
                ExitCode::from(2)
            }
        };
    }
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    match outline_with(path, &src, symbols, form, expanded) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(mut diags) => {
            diag::report(&mut diags, path, &src, json);
            ExitCode::from(2)
        }
    }
}

/// The outline of `src` as JSON (or text), or the diagnostics that stopped
/// it: a rejected program has no reliable outline (api.out.5).
#[cfg(test)]
pub fn outline(path: &str, src: &str, symbols: bool, form: Form) -> Result<String, Vec<diag::Diagnostic>> {
    outline_with(path, src, symbols, form, false)
}

/// `outline`; with `expanded`, also the code derive generators wrote
/// (ct.gen.4) and which `impl`s are built-in derives (`api/expanded.rs`).
pub fn outline_with(
    path: &str,
    src: &str,
    symbols: bool,
    form: Form,
    expanded: bool,
) -> Result<String, Vec<diag::Diagnostic>> {
    // The file and the modules it imports (mod.unit.3); only the file's own
    // declarations are outlined.
    let (mut prog, mut diags) = crate::modules::load_src(path, src);
    let mut resolved = Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&prog);
    }
    // A library file has no `main`, and still has an outline (api.out.5).
    diags.retain(|d| d.rule != "mod.main.1");
    if diags.iter().any(|d| !d.warning) {
        return Err(diags);
    }
    docs::attach(&mut prog, src);
    let effects = check::effects::effects_of(&prog, &resolved);
    let mut api = Api::new(&prog, effects);
    api.expanded = expanded;
    api.errors = std::mem::take(&mut resolved.fn_errors);
    if form == Form::Text {
        return Ok(api.text());
    }
    let mut top = vec![("api", Json::Int(API_VERSION)), ("file", Json::Str(path.to_string())), ("items", api.items())];
    if symbols {
        top.push(("references", api.references(&resolved)));
    }
    let mut out = String::new();
    Json::Obj(top).write(&mut out, (form == Form::Pretty).then_some(0));
    out.push('\n');
    Ok(out)
}

/// Every reference in the program as (file, line in that file, column, id),
/// in position order; ids carry their module in a package (api.ref.1, api.pkg.1).
pub fn references(prog: &Program, resolved: &Resolved, package: bool) -> Vec<(String, usize, usize, String)> {
    let mut refs = Refs { prog, resolved, found: Vec::new(), generics: Vec::new(), names: Vec::new() };
    refs.program();
    let mut found = refs.found;
    found.retain(|(p, _)| !crate::ast::is_prelude(*p) && (package || !is_prelude(*p)));
    found.sort_by(|a, b| (a.0.line, a.0.col, &a.1).cmp(&(b.0.line, b.0.col, &b.1)));
    found.dedup();
    found
        .into_iter()
        .map(|(p, id)| match prog.file_at(p) {
            Some(f) => (f.path.clone(), p.line - f.base, p.col, id),
            None => (String::new(), p.line, p.col, id),
        })
        .collect()
}

enum PackageFailure {
    Message(String),
    Diags(Vec<crate::ast::SourceFile>, Vec<diag::Diagnostic>),
}

/// The outline of the package directory `dir` (api.pkg.1): every module's
/// declarations in one list, ordered by file then position. An item's `id`
/// is prefixed with its module (`fn:net.get`, `type:ui.Window`), and it gains
/// `module` and `file` keys after the single-file ones; `line` counts from
/// the top of its own file. `--symbols` adds `references` with a `file` key.
fn package_outline(dir: &str, symbols: bool, form: Form) -> Result<String, PackageFailure> {
    let (mut prog, mut diags) = modules::load(dir).map_err(PackageFailure::Message)?;
    let mut resolved = Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&prog);
    }
    diags.retain(|d| d.rule != "mod.main.1");
    if diags.iter().any(|d| !d.warning) {
        return Err(PackageFailure::Diags(prog.files, diags));
    }
    let mut by_line = HashMap::new();
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
    let mut items = api.outline_items();
    items.sort_by_key(|i| (i.pos.line, i.pos.col));
    for i in &mut items {
        localize(&prog, i);
    }
    if form == Form::Text {
        let mut out = String::new();
        let mut file = String::new();
        for i in &items {
            let here = i.extra.iter().find_map(|(k, v)| if *k == "file" { if let Json::Str(s) = v { Some(s.as_str()) } else { None } } else { None });
            if here.is_some_and(|h| h != file) {
                file = here.unwrap_or_default().to_string();
                writeln!(out, "# {file}").unwrap();
            }
            i.text(0, &mut out);
        }
        return Ok(out);
    }
    let mut top = vec![
        ("api", Json::Int(API_VERSION)),
        ("package", Json::Str(dir.to_string())),
        ("items", Json::Arr(items.into_iter().map(Item::json).collect())),
    ];
    if symbols {
        let mut refs = Refs { prog: &prog, resolved: &resolved, found: Vec::new(), generics: Vec::new(), names: Vec::new() };
        refs.program();
        let mut found = refs.found;
        found.retain(|(p, _)| !crate::ast::is_prelude(*p));
        found.sort_by(|a, b| (a.0.line, a.0.col, &a.1).cmp(&(b.0.line, b.0.col, &b.1)));
        found.dedup();
        let list = found.into_iter().map(|(p, id)| {
            let (file, line) = match prog.file_at(p) {
                Some(f) => (f.path.clone(), p.line - f.base),
                None => (String::new(), p.line),
            };
            Json::Obj(vec![
                ("id", Json::Str(id)),
                ("line", Json::Int(line as i64)),
                ("col", Json::Int(p.col as i64)),
                ("file", Json::Str(file)),
            ])
        });
        top.push(("references", Json::Arr(list.collect())));
    }
    let mut out = String::new();
    Json::Obj(top).write(&mut out, (form == Form::Pretty).then_some(0));
    out.push('\n');
    Ok(out)
}

/// Make an item's lines count from the top of its file, and say which file
/// and module it is in (api.pkg.1).
fn localize(prog: &Program, item: &mut Item) {
    if let Some(f) = prog.file_at(item.pos) {
        item.extra.push(("module", Json::Str(f.module.clone())));
        item.extra.push(("file", Json::Str(f.path.clone())));
        shift_lines(item, f.base);
    }
}

fn shift_lines(item: &mut Item, base: usize) {
    item.pos.line = item.pos.line.saturating_sub(base);
    for (_, ms) in &mut item.members {
        for m in ms {
            shift_lines(m, base);
        }
    }
}

/// A JSON value whose objects keep their keys in the order given (api.out.2).
enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(&'static str, Json)>),
}

impl Json {
    fn opt(s: Option<String>) -> Json {
        s.map_or(Json::Null, Json::Str)
    }

    /// With `depth`, pretty-printed with a two-space indent; without, on
    /// one line with no spaces. An empty array is `[]` either way.
    fn write(&self, out: &mut String, depth: Option<usize>) {
        let newline = |out: &mut String, d: usize| {
            if depth.is_some() {
                out.push('\n');
                out.extend(std::iter::repeat_n("  ", d));
            }
        };
        let inner = depth.map(|d| d + 1);
        let d = depth.unwrap_or(0);
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Int(n) => write!(out, "{n}").unwrap(),
            Json::Str(s) => out.push_str(&diag::json_str(s)),
            Json::Arr(xs) if xs.is_empty() => out.push_str("[]"),
            Json::Arr(xs) => {
                out.push('[');
                for (i, x) in xs.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    newline(out, d + 1);
                    x.write(out, inner);
                }
                newline(out, d);
                out.push(']');
            }
            Json::Obj(kvs) => {
                out.push('{');
                for (i, (k, v)) in kvs.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    newline(out, d + 1);
                    write!(out, "\"{k}\":{}", if depth.is_some() { " " } else { "" }).unwrap();
                    v.write(out, inner);
                }
                newline(out, d);
                out.push('}');
            }
        }
    }
}

/// A declaration in the outline, before it is written as JSON or text.
struct Item {
    pos: Pos,
    kind: &'static str,
    name: String,
    id: String,
    doc: Option<String>,
    signature: String,
    /// Keys after `signature`, in order.
    extra: Vec<(&'static str, Json)>,
    /// Fields, variants or methods, each list under its key.
    members: Vec<(&'static str, Vec<Item>)>,
}

impl Item {
    fn json(self) -> Json {
        let mut kvs = vec![
            ("kind", Json::Str(self.kind.to_string())),
            ("name", Json::Str(self.name)),
            ("id", Json::Str(self.id)),
            ("line", Json::Int(self.pos.line as i64)),
            ("col", Json::Int(self.pos.col as i64)),
            ("doc", Json::opt(self.doc)),
            ("signature", Json::Str(self.signature)),
        ];
        kvs.extend(self.extra);
        for (key, items) in self.members {
            kvs.push((key, Json::Arr(items.into_iter().map(Item::json).collect())));
        }
        Json::Obj(kvs)
    }

    fn text(&self, depth: usize, out: &mut String) {
        let indent = "    ".repeat(depth);
        writeln!(out, "{:>5}  {indent}{}", self.pos.line, self.signature).unwrap();
        // Contracts are part of the signature (contract.api.1).
        for (key, value) in self.extra.iter() {
            let (word, from) = match *key {
                "requires" | "ensures" => (*key, ""),
                "inherited_requires" => ("requires", " (inherited)"),
                "inherited_ensures" => ("ensures", " (inherited)"),
                "invariants" => ("invariant", ""),
                _ => continue,
            };
            if let Json::Arr(conds) = value {
                for c in conds {
                    if let Json::Str(c) = c {
                        writeln!(out, "       {indent}    {word} {c}{from}").unwrap();
                    }
                }
            }
        }
        // err.api.1: an inferred error set is spelled out.
        if self.signature.contains(") !")
            && let Some((_, Json::Arr(errs))) = self.extra.iter().find(|(k, _)| *k == "fails_with")
        {
            let names: Vec<&str> = errs.iter().filter_map(|e| if let Json::Str(s) = e { Some(s.as_str()) } else { None }).collect();
            writeln!(out, "       {indent}    fails with {}", names.join(" | ")).unwrap();
        }
        // ct.gen.4: `--expanded` shows generated code under its signature.
        if let Some((_, Json::Str(code))) = self.extra.iter().find(|(k, _)| *k == "text") {
            for line in code.lines().skip(1) {
                writeln!(out, "       {indent}{line}").unwrap();
            }
        }
        if let Some(doc) = &self.doc {
            for line in doc.lines() {
                writeln!(out, "       {indent}    ## {line}").unwrap();
            }
        }
        // A variant's signature already shows its fields.
        for (_, members) in self.members.iter().filter(|_| self.kind != "variant") {
            for m in members {
                m.text(depth + 1, out);
            }
        }
    }
}

struct Api<'p> {
    prog: &'p Program,
    /// The id of each function, by index into `Program::fns` (api.id.1-2).
    fn_ids: Vec<String>,
    /// Each function's checked effect set, by the effects lane's id (api.out.4).
    effects: std::collections::BTreeMap<String, std::collections::BTreeSet<&'static str>>,
    /// The error types of each fallible function, by function index (err.api.1).
    errors: std::collections::BTreeMap<usize, Vec<String>>,
    /// Also describe the modules the file imports (`--refs`).
    whole: bool,
    /// Include generated code (`--expanded`, ct.gen.4).
    expanded: bool,
}

impl<'p> Api<'p> {
    fn new(prog: &'p Program, effects: std::collections::BTreeMap<String, std::collections::BTreeSet<&'static str>>) -> Self {
        let mut fn_ids: Vec<String> = prog.fns.iter().map(|f| fn_id(prog, f, false)).collect();
        // api.id.2: an `impl` method whose plain id is taken also names its trait.
        let mut seen: HashMap<String, usize> = HashMap::new();
        for id in &fn_ids {
            *seen.entry(id.clone()).or_default() += 1;
        }
        for (i, f) in prog.fns.iter().enumerate() {
            if matches!(f.within, Within::Impl(_)) && seen[&fn_ids[i]] > 1 {
                fn_ids[i] = fn_id(prog, f, true);
            }
        }
        Api { prog, fn_ids, effects, errors: std::collections::BTreeMap::new(), whole: false, expanded: false }
    }

    /// Whether `pos` is a declaration the outline leaves out.
    fn outside(&self, pos: Pos) -> bool {
        if self.whole { crate::ast::is_prelude(pos) } else { is_prelude(pos) }
    }

    /// Every declaration of the program (not the prelude), nested: fields,
    /// variants and inherent methods under their type, methods under their
    /// trait or `impl` (api.out.3). Sorted by position.
    fn outline_items(&self) -> Vec<Item> {
        let prog = self.prog;
        let mut methods: HashMap<&str, Vec<Item>> = HashMap::new();
        let mut items = Vec::new();
        let local_types: HashSet<&str> =
            prog.structs.iter().map(|s| s.name.as_str()).chain(prog.enums.iter().map(|e| e.name.as_str())).collect();
        for (i, f) in prog.fns.iter().enumerate() {
            if self.outside(f.pos) || !f.top_level() {
                continue;
            }
            match &f.owner {
                Some(o) if local_types.contains(o.name.as_str()) => {
                    methods.entry(o.name.as_str()).or_default().push(self.fn_item(i));
                }
                _ => items.push(self.fn_item(i)),
            }
        }
        let mut take = |name: &str| {
            let mut ms = methods.remove(name).unwrap_or_default();
            ms.sort_by_key(|m| (m.pos.line, m.pos.col));
            ms
        };
        for s in prog.structs.iter().filter(|s| !self.outside(s.pos)) {
            let fields = s.fields.iter().map(|f| field_item(&s.name, f)).collect();
            items.push(Item {
                pos: s.pos,
                kind: "struct",
                name: s.name.clone(),
                id: format!("type:{}", s.name),
                doc: s.doc.clone(),
                signature: format!(
                    "{}{}struct {}{}",
                    error_prefix(s.is_error),
                    unique_prefix(s.unique),
                    s.name,
                    generics_text(&s.generics)
                ),
                extra: {
                    let mut extra = unique_extra(s.is_error, s.unique);
                    // contract.api.2: the invariants as written.
                    if !s.invariants.is_empty() {
                        extra.push(("invariants", Json::Arr(s.invariants.iter().map(|c| Json::Str(c.text.clone())).collect())));
                    }
                    extra
                },
                members: vec![("fields", fields), ("methods", take(&s.name))],
            });
        }
        for e in prog.enums.iter().filter(|e| !self.outside(e.pos)) {
            let variants = e.variants.iter().map(|v| variant_item(&e.name, v)).collect();
            items.push(Item {
                pos: e.pos,
                kind: "enum",
                name: e.name.clone(),
                id: format!("type:{}", e.name),
                doc: e.doc.clone(),
                signature: format!(
                    "{}{}enum {}{}",
                    error_prefix(e.is_error),
                    unique_prefix(e.unique),
                    e.name,
                    generics_text(&e.generics)
                ),
                extra: unique_extra(e.is_error, e.unique),
                members: vec![("variants", variants), ("methods", take(&e.name))],
            });
        }
        for t in prog.traits.iter().filter(|t| !self.outside(t.pos)) {
            items.push(Item {
                pos: t.pos,
                kind: "trait",
                name: t.name.clone(),
                id: format!("trait:{}", t.name),
                doc: t.doc.clone(),
                signature: format!("trait {}", t.name),
                extra: Vec::new(),
                members: vec![("methods", t.methods.iter().map(|&m| self.fn_item(m)).collect())],
            });
        }
        for im in prog.impls.iter().filter(|im| !self.outside(im.pos)) {
            let target = owner_text(&im.target);
            items.push(Item {
                pos: im.pos,
                kind: "impl",
                name: format!("{} for {}", im.trait_name, im.target.name),
                id: format!("impl:{}.{}", im.target.name, im.trait_name),
                doc: None,
                signature: format!("impl {} for {target}", im.trait_name),
                extra: [
                    ("trait", Json::Str(format!("trait:{}", im.trait_name))),
                    ("type", Json::Str(format!("type:{}", im.target.name))),
                ]
                .into_iter()
                .chain(self.derive_extra(im))
                .collect(),
                members: vec![("methods", self.impl_methods(im))],
            });
        }
        for c in prog.consts.iter().filter(|c| !self.outside(c.pos)) {
            items.push(Item {
                pos: c.pos,
                kind: "const",
                name: c.name.clone(),
                id: format!("const:{}", c.name),
                doc: c.doc.clone(),
                signature: format!("{} := {}", c.name, const_value(&c.value)),
                extra: Vec::new(),
                members: Vec::new(),
            });
        }
        // Methods of a type this file doesn't declare stay at the top level.
        items.extend(methods.into_values().flatten());
        items.sort_by_key(|i| (i.pos.line, i.pos.col));
        items
    }

    fn items(&self) -> Json {
        Json::Arr(self.outline_items().into_iter().map(Item::json).collect())
    }

    /// The text outline: one signature per line after its line number,
    /// members indented, doc comments under their declaration (api.text.1).
    fn text(&self) -> String {
        let mut out = String::new();
        for item in self.outline_items() {
            item.text(0, &mut out);
        }
        out
    }

    fn fn_item(&self, i: usize) -> Item {
        let f = &self.prog.fns[i];
        let params = f
            .params
            .iter()
            .map(|p| {
                let mut fields = vec![
                    ("name", Json::Str(p.name.clone())),
                    ("type", Json::Str(type_text(&p.ty))),
                    ("mut", Json::Bool(p.mutable)),
                ];
                // Only a `move` parameter says so (own.move.1), to keep the common case short.
                if p.moved {
                    fields.push(("move", Json::Bool(true)));
                }
                Json::Obj(fields)
            })
            .collect();
        let method = owner_name(self.prog, f).is_some();
        let mut extra = Vec::new();
        if method {
            let receiver = f.receiver.map(|r| receiver_text(r).to_string());
            extra.push(("receiver", Json::opt(receiver)));
        }
        extra.push(("params", Json::Arr(params)));
        extra.push(("returns", Json::opt(f.ret.as_ref().map(type_text))));
        // err.api.1: the error types a fallible function can fail with.
        if let Some(errors) = self.errors.get(&i) {
            extra.push(("fails_with", Json::Arr(errors.iter().cloned().map(Json::Str).collect())));
        }
        extra.push(("effects", Json::Arr(self.effects(f))));
        // contract.api.1: the conditions as written, when there are any.
        for (key, ensures) in [("requires", false), ("ensures", true)] {
            let conds: Vec<Json> =
                f.contracts.iter().filter(|c| c.ensures == ensures).map(|c| Json::Str(c.text.clone())).collect();
            if !conds.is_empty() {
                extra.push((key, Json::Arr(conds)));
            }
        }
        // contract.api.2: what an implementation inherits from its trait's method.
        if let Some(t) = f.inherits.map(|m| &self.prog.fns[m]) {
            if let Within::Trait(k) = t.within {
                extra.push(("inherits", Json::Str(format!("{}.{}", self.prog.traits[k].name, t.name))));
            }
            for (key, ensures) in [("inherited_requires", false), ("inherited_ensures", true)] {
                let conds: Vec<Json> =
                    t.contracts.iter().filter(|c| c.ensures == ensures).map(|c| Json::Str(c.text.clone())).collect();
                if !conds.is_empty() {
                    extra.push((key, Json::Arr(conds)));
                }
            }
        }
        if matches!(f.within, Within::Trait(_)) {
            extra.push(("default", Json::Bool(f.body.is_some())));
        }
        Item {
            pos: f.pos,
            kind: if method { "method" } else { "fn" },
            name: f.name.clone(),
            id: self.fn_ids[i].clone(),
            doc: f.doc.clone(),
            signature: fn_signature(f),
            extra,
            members: Vec::new(),
        }
    }

    /// Every use of a declared name, keyed by the id of what it refers to
    /// and sorted by position (api.ref.1-2).
    fn references(&self, resolved: &Resolved) -> Json {
        let mut refs = Refs { prog: self.prog, resolved, found: Vec::new(), generics: Vec::new(), names: Vec::new() };
        refs.program();
        let mut found = refs.found;
        found.retain(|(p, _)| !is_prelude(*p));
        found.sort_by(|a, b| (a.0.line, a.0.col, &a.1).cmp(&(b.0.line, b.0.col, &b.1)));
        found.dedup();
        Json::Arr(
            found
                .into_iter()
                .map(|(p, id)| {
                    Json::Obj(vec![
                        ("id", Json::Str(id)),
                        ("line", Json::Int(p.line as i64)),
                        ("col", Json::Int(p.col as i64)),
                    ])
                })
                .collect(),
        )
    }
}

impl<'p> Api<'p> {
    /// The effects a function performs (api.out.4, eff.infer.1), from the
    /// checked effect set, sorted.
    fn effects(&self, f: &FnDecl) -> Vec<Json> {
        let id = check::effects::fn_id(self.prog, f);
        self.effects.get(&id).map(|set| set.iter().map(|e| Json::Str(e.to_string())).collect()).unwrap_or_default()
    }
}

/// `fn:name`, `method:Type.name`, `method:Trait.name`, or with `qualified`,
/// `method:Type.Trait.name` for an `impl` method (api.id.1-2).
fn fn_id(prog: &Program, f: &FnDecl, qualified: bool) -> String {
    match (owner_name(prog, f), f.within) {
        (Some(o), Within::Impl(i)) if qualified => format!("method:{o}.{}.{}", prog.impls[i].trait_name, f.name),
        (Some(o), _) => format!("method:{o}.{}", f.name),
        (None, _) => format!("fn:{}", f.name),
    }
}

/// The type or trait a function belongs to: its `Type.` at the top level,
/// or its `trait` or `impl` block's.
fn owner_name<'p>(prog: &'p Program, f: &'p FnDecl) -> Option<&'p str> {
    match f.within {
        Within::Trait(t) => Some(&prog.traits[t].name),
        Within::Impl(i) => Some(&prog.impls[i].target.name),
        Within::TopLevel => f.owner.as_ref().map(|o| o.name.as_str()),
    }
}

fn field_item(owner: &str, f: &Field) -> Item {
    let ty = type_text(&f.ty);
    Item {
        pos: f.pos,
        kind: "field",
        name: f.name.clone(),
        id: format!("field:{owner}.{}", f.name),
        doc: f.doc.clone(),
        signature: format!("{} {ty}", f.name),
        extra: vec![("type", Json::Str(ty))],
        members: Vec::new(),
    }
}

fn variant_item(owner: &str, v: &Variant) -> Item {
    let fields: Vec<String> = v.fields.iter().map(|f| format!("{} {}", f.name, type_text(&f.ty))).collect();
    let signature = if fields.is_empty() { v.name.clone() } else { format!("{}({})", v.name, fields.join(", ")) };
    let path = format!("{owner}.{}", v.name);
    let fields = v.fields.iter().map(|f| field_item(&path, f)).collect();
    Item {
        pos: v.pos,
        kind: "variant",
        name: v.name.clone(),
        id: format!("variant:{path}"),
        doc: v.doc.clone(),
        signature,
        extra: Vec::new(),
        members: vec![("fields", fields)],
    }
}

/// `error`, and `unique` only for a unique type (own.uniq.1).
fn unique_extra(is_error: bool, unique: bool) -> Vec<(&'static str, Json)> {
    let mut extra = vec![("error", Json::Bool(is_error))];
    if unique {
        extra.push(("unique", Json::Bool(true)));
    }
    extra
}

fn unique_prefix(unique: bool) -> &'static str {
    if unique { "unique " } else { "" }
}

fn error_prefix(is_error: bool) -> &'static str {
    if is_error { "error " } else { "" }
}

/// `fn name[T Ord](a int, mut b List[T]) T ! E`, as written; a top-level
/// method keeps its `Type.` (api.out.3).
fn fn_signature(f: &FnDecl) -> String {
    let mut s = String::from("fn ");
    if let (Some(o), true) = (&f.owner, f.top_level()) {
        write!(s, "{}.", owner_text(o)).unwrap();
    }
    s.push_str(&f.name);
    s.push_str(&generics_text(&f.generics));
    let mut parts: Vec<String> = Vec::new();
    if let Some(r) = f.receiver {
        parts.push(receiver_text(r).to_string());
    }
    for p in &f.params {
        let mode = if p.mutable {
            "mut "
        } else if p.moved {
            "move "
        } else {
            ""
        };
        parts.push(format!("{mode}{} {}", p.name, type_text(&p.ty)));
    }
    write!(s, "({})", parts.join(", ")).unwrap();
    if let Some(t) = &f.ret {
        write!(s, " {}", type_text(t)).unwrap();
    }
    s
}

fn owner_text(o: &Owner) -> String {
    match &o.params {
        Some(ps) => format!("{}{}", o.name, generics_text(ps)),
        None => o.name.clone(),
    }
}

/// `[T Ord + Eq, U]`, or nothing.
fn generics_text(gs: &[GenericParam]) -> String {
    if gs.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = gs
        .iter()
        .map(|g| {
            let bounds: Vec<&str> = g.bounds.iter().map(|(b, _)| b.as_str()).collect();
            if bounds.is_empty() { g.name.clone() } else { format!("{} {}", g.name, bounds.join(" + ")) }
        })
        .collect();
    format!("[{}]", parts.join(", "))
}

/// A type as written in source (type.opt.2, err.type.1-2, err.type.5, trait.any.1).
pub(crate) fn type_text(t: &TypeRef) -> String {
    match &t.kind {
        TypeKind::Named { name, args } if args.is_empty() => name.clone(),
        TypeKind::Named { name, args } => {
            let args: Vec<String> = args.iter().map(type_text).collect();
            format!("{name}[{}]", args.join(", "))
        }
        TypeKind::Optional(t) => format!("{}?", type_text(t)),
        TypeKind::Any(name) => format!("any {name}"),
        TypeKind::Ref(t) => format!("ref {}", type_text(t)),
        TypeKind::Fallible { ok, errors } => {
            let errs = errors.as_ref().map(|es| es.iter().map(type_text).collect::<Vec<_>>().join(" | "));
            match (ok, errs) {
                (Some(ok), Some(errs)) => format!("{} ! {errs}", type_text(ok)),
                (Some(ok), None) => format!("!{}", type_text(ok)),
                (None, Some(errs)) => format!("! {errs}"),
                (None, None) => "!".to_string(),
            }
        }
    }
}

/// A constant's value when it is a literal, else `...`: the outline
/// leaves out bodies (api.out.3).
fn const_value(e: &Expr) -> String {
    match e {
        Expr::Int(n, _) => n.to_string(),
        Expr::Float(x, _) => format!("{x:?}"),
        Expr::Bool(b, _) => b.to_string(),
        Expr::Char(c, _) => format!("{c:?}"),
        Expr::Str(s, _) => diag::json_str(s),
        Expr::Unary { op: UnOp::Neg, operand, .. } if matches!(**operand, Expr::Int(..) | Expr::Float(..)) => {
            format!("-{}", const_value(operand))
        }
        _ => "...".to_string(),
    }
}

/// The reference pass (api.ref.1-2): walks the program, taking what each use
/// refers to from the checker's `Resolved` facts, and type mentions from
/// the declared names.
struct Refs<'p> {
    prog: &'p Program,
    resolved: &'p Resolved,
    found: Vec<(Pos, String)>,
    /// Type parameter names in scope, which shadow nothing but aren't references.
    generics: Vec<String>,
    /// Names the checker did not resolve to a declaration: locals (api.refs.3).
    names: Vec<(Pos, String)>,
}

impl<'p> Refs<'p> {
    fn add(&mut self, pos: Pos, id: String) {
        self.found.push((pos, id));
    }

    fn program(&mut self) {
        let prog = self.prog;
        for s in &prog.structs {
            self.generics = s.generics.iter().map(|g| g.name.clone()).collect();
            self.generic_bounds(&s.generics);
            for f in &s.fields {
                self.type_ref(&f.ty);
            }
            for c in &s.invariants {
                self.expr(&c.cond);
            }
        }
        for e in &prog.enums {
            self.generics = e.generics.iter().map(|g| g.name.clone()).collect();
            self.generic_bounds(&e.generics);
            for v in &e.variants {
                for f in &v.fields {
                    self.type_ref(&f.ty);
                }
            }
        }
        self.generics.clear();
        for im in &prog.impls {
            self.add(im.trait_pos, format!("trait:{}", im.trait_name));
            self.owner(&im.target);
        }
        for f in &prog.fns {
            self.generics.clear();
            if let Within::Impl(i) = f.within {
                self.scope_owner(&prog.impls[i].target);
            }
            if let Some(o) = &f.owner {
                if f.top_level() {
                    self.owner(o);
                } else {
                    self.scope_owner(o);
                }
            }
            self.generics.extend(f.generics.iter().map(|g| g.name.clone()));
            self.generic_bounds(&f.generics);
            for p in &f.params {
                self.type_ref(&p.ty);
            }
            if let Some(t) = &f.ret {
                self.type_ref(t);
            }
            for c in &f.contracts {
                self.expr(&c.cond);
                for (_, e) in &c.olds {
                    self.expr(e);
                }
            }
            if let Some(body) = &f.body {
                self.block(body);
            }
        }
        for c in &prog.consts {
            self.expr(&c.value);
        }
    }

    /// The type parameters an owner brings into scope, from its declaration.
    fn scope_owner(&mut self, o: &Owner) {
        let prog = self.prog;
        let declared = prog
            .structs
            .iter()
            .find(|s| s.name == o.name)
            .map(|s| &s.generics)
            .or_else(|| prog.enums.iter().find(|e| e.name == o.name).map(|e| &e.generics));
        if let Some(gs) = declared {
            self.generics.extend(gs.iter().map(|g| g.name.clone()));
        }
        if let Some(ps) = &o.params {
            self.generics.extend(ps.iter().map(|g| g.name.clone()));
        }
    }

    /// `Type` in `fn Type.name` or `impl Trait for Type`, with its bounds.
    fn owner(&mut self, o: &Owner) {
        if let Some(id) = self.type_id(&o.name) {
            self.add(o.pos, id);
        }
        self.scope_owner(o);
        if let Some(ps) = &o.params {
            self.generic_bounds(ps);
        }
    }

    fn generic_bounds(&mut self, gs: &[GenericParam]) {
        for g in gs {
            for (b, p) in &g.bounds {
                self.add(*p, format!("trait:{b}"));
            }
        }
    }

    /// The id of a declared type or trait called `name`, unless a type
    /// parameter in scope has that name.
    fn type_id(&self, name: &str) -> Option<String> {
        if self.generics.iter().any(|g| g == name) {
            return None;
        }
        let prog = self.prog;
        if prog.structs.iter().any(|s| s.name == name) || prog.enums.iter().any(|e| e.name == name) {
            Some(format!("type:{name}"))
        } else if prog.traits.iter().any(|t| t.name == name) {
            Some(format!("trait:{name}"))
        } else {
            None
        }
    }

    fn type_ref(&mut self, t: &TypeRef) {
        match &t.kind {
            TypeKind::Named { name, args } => {
                if let Some(id) = self.type_id(name) {
                    self.add(t.pos, id);
                }
                for a in args {
                    self.type_ref(a);
                }
            }
            TypeKind::Optional(t) | TypeKind::Ref(t) => self.type_ref(t),
            TypeKind::Any(name) => self.add(t.pos, format!("trait:{name}")),
            TypeKind::Fallible { ok, errors } => {
                if let Some(ok) = ok {
                    self.type_ref(ok);
                }
                for e in errors.iter().flatten() {
                    self.type_ref(e);
                }
            }
        }
    }

    fn block(&mut self, b: &Block) {
        for s in b {
            self.stmt(s);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let { value, otherwise, .. } => {
                self.expr(value);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            Stmt::Assign { target, value, .. } => {
                self.expr(target);
                self.expr(value);
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::For { iter, body, .. } | Stmt::With { mutex: iter, body, .. } => {
                self.expr(iter);
                self.block(body);
            }
            Stmt::Unsafe { body, .. } => self.block(body),
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
            Stmt::Return(Some(e), _) | Stmt::Expr(e) => self.expr(e),
            Stmt::Handle { value, otherwise } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Stmt::Defer(s, _) => self.stmt(s),
        }
    }

    fn otherwise(&mut self, o: &Else) {
        match o {
            Else::Value(e) => self.expr(e),
            Else::Leave(s) => self.stmt(s),
            Else::Block { body, .. } => self.block(body),
        }
    }

    /// The id the checker resolved for the use at `pos`, if any.
    fn resolved_at(&self, pos: Pos, name: &str) -> Option<String> {
        if let Some(id) = self.resolved.symbols.get(&pos) {
            return Some(id.clone());
        }
        if let Some(&(e, v)) = self.resolved.variants.get(&pos) {
            let decl = &self.prog.enums[e];
            return Some(format!("variant:{}.{}", decl.name, decl.variants[v].name));
        }
        match self.resolved.calls.get(&pos) {
            Some(Call::Trait(t) | Call::Qualified(t)) => Some(format!("method:{}.{name}", self.prog.traits[*t].name)),
            None => None,
        }
    }

    /// `Type` or `Trait` before `.name` in `Type.f(...)`, `Enum.Variant` or
    /// `Trait.f(x)`, when `.name` resolved through it.
    fn qualifier(&mut self, recv: &Expr, member: Option<&str>) {
        let (Expr::Name(n, p) | Expr::TypeArgs { name: n, pos: p, .. }) = recv else { return };
        let Some(member) = member else { return };
        let owner = member.split_once(':').map_or("", |(_, path)| path.split('.').next().unwrap_or(""));
        if owner == n
            && let Some(id) = self.type_id(n)
        {
            self.add(*p, id);
        }
    }

    fn args(&mut self, args: &[Arg], fields_of: Option<String>) {
        for a in args {
            if let (Some((label, p)), Some(owner)) = (&a.label, &fields_of) {
                self.add(*p, format!("field:{owner}.{label}"));
            }
            self.expr(&a.value);
        }
    }

    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Str(..) | Expr::Int(..) | Expr::Float(..) | Expr::Char(..) | Expr::Bool(..) | Expr::None(_) | Expr::Hole(_) => {}
            Expr::Interp(parts, _) => {
                for p in parts {
                    if let InterpPart::Expr(e) | InterpPart::Fmt(e, _) = p {
                        self.expr(e);
                    }
                }
            }
            Expr::Name(n, p) => {
                if let Some(id) = self.resolved_at(*p, n) {
                    self.add(*p, id);
                } else {
                    self.names.push((*p, n.clone()));
                }
            }
            Expr::TypeArgs { name, args, pos } => {
                if let Some(id) = self.resolved_at(*pos, name) {
                    self.add(*pos, id);
                }
                for a in args {
                    self.type_ref(a);
                }
            }
            Expr::List(xs, _) | Expr::Set(xs, _) => {
                for x in xs {
                    self.expr(x);
                }
            }
            Expr::Comp { elem, value, clauses, .. } => {
                for c in clauses {
                    self.expr(&c.iter);
                    if let Some(cond) = &c.cond {
                        self.expr(cond);
                    }
                }
                self.expr(elem);
                if let Some(v) = value {
                    self.expr(v);
                }
            }
            Expr::Map(kvs, _) => {
                for (k, v) in kvs {
                    self.expr(k);
                    self.expr(v);
                }
            }
            Expr::Call { callee, args, .. } => {
                self.expr(callee);
                let id = match &**callee {
                    Expr::Name(n, p) | Expr::TypeArgs { name: n, pos: p, .. } => self.resolved_at(*p, n),
                    _ => None,
                };
                self.args(args, id.and_then(|id| fields_owner(&id)));
            }
            Expr::Method { recv, name, name_pos, args } => {
                let id = self.resolved_at(*name_pos, name);
                self.qualifier(recv, id.as_deref());
                self.expr(recv);
                let fields = id.as_deref().and_then(fields_owner);
                if let Some(id) = id {
                    self.add(*name_pos, id);
                }
                self.args(args, fields);
            }
            Expr::Field { recv, name, name_pos } => {
                let id = self.resolved_at(*name_pos, name);
                self.qualifier(recv, id.as_deref());
                self.expr(recv);
                if let Some(id) = id {
                    self.add(*name_pos, id);
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
                for x in lo.iter().chain(hi.iter()) {
                    self.expr(x);
                }
            }
            Expr::If { branches, els, .. } => {
                for (cond, body) in branches {
                    match cond {
                        Cond::Expr(e) | Cond::Bind { value: e, .. } => self.expr(e),
                    }
                    self.block(body);
                }
                if let Some(b) = els {
                    self.block(b);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                for arm in arms {
                    self.pattern(&arm.pat);
                    if let Some(g) = &arm.guard {
                        self.expr(g);
                    }
                    self.block(&arm.body);
                }
            }
            Expr::Try { expr, context, .. } => {
                self.expr(expr);
                if let Some(c) = context {
                    self.expr(c);
                }
            }
            Expr::Else { value, otherwise, .. } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
        }
    }

    fn pattern(&mut self, p: &Pattern) {
        match p {
            Pattern::Wildcard(_) | Pattern::Literal(..) | Pattern::Range { .. } => {}
            Pattern::Or(ps, _) => {
                for p in ps {
                    self.pattern(p);
                }
            }
            Pattern::Variant { qualifier, pos, .. } => {
                if let Some(id) = self.resolved.symbols.get(pos).cloned() {
                    if let Some(q) = qualifier
                        && let Some(tid) = self.type_id(q)
                    {
                        self.add(*pos, tid);
                    }
                    self.add(*pos, id);
                }
            }
        }
    }
}

/// The owner path of the fields a struct literal or variant sets: `Point`
/// for `type:Point`, `Shape.Rect` for `variant:Shape.Rect`.
fn fields_owner(id: &str) -> Option<String> {
    id.strip_prefix("type:").or_else(|| id.strip_prefix("variant:")).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example(name: &str) -> (String, String) {
        let path = format!("{}/../examples/{name}", env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(&path).unwrap();
        (path, src)
    }

    fn api_of(src: &str, symbols: bool) -> String {
        outline("t.ail", src, symbols, Form::Pretty).unwrap_or_else(|d| panic!("{d:?}"))
    }

    #[test]
    fn shapes_outline_has_key_items() {
        let (path, src) = example("shapes.ail");
        let out = outline(&path, &src, false, Form::Pretty).unwrap();
        assert!(out.starts_with("{\n  \"api\": 1,\n"), "{out}");
        for want in [
            "\"id\": \"trait:Shape\"",
            "\"signature\": \"fn describe(self) str\"",
            "\"default\": true",
            "\"id\": \"type:Circle\"",
            "\"id\": \"field:Rect.h\"",
            "\"id\": \"impl:Circle.Shape\"",
            "\"id\": \"method:Circle.area\"",
            "\"id\": \"method:Drawing.add\"",
            "\"signature\": \"fn Drawing.add(mut self, s any Shape)\"",
            "\"signature\": \"fn Drawing.biggest(self) Size?\"",
            "\"signature\": \"fn largest[T Ord](items List[T]) T\"",
            "\"id\": \"fn:main\"",
            "\"effects\": []",
        ] {
            assert!(out.contains(want), "missing {want}\n{out}");
        }
        // Methods sit under their type: `Drawing.add` comes after `field:Drawing.shapes`.
        let field = out.find("\"id\": \"field:Drawing.shapes\"").unwrap();
        let method = out.find("\"id\": \"method:Drawing.add\"").unwrap();
        assert!(field < method);
        assert!(!out.contains("\"references\""));
    }

    #[test]
    fn shapes_symbols_resolve_uses() {
        let (path, src) = example("shapes.ail");
        let out = outline(&path, &src, true, Form::Pretty).unwrap();
        let refs = &out[out.find("\"references\"").unwrap()..];
        for want in [
            // `s.area()` in `Drawing.biggest` goes through the trait.
            "\"id\": \"method:Shape.area\",\n      \"line\": 48",
            "\"id\": \"field:Drawing.shapes\",\n      \"line\": 44",
            "\"id\": \"type:Size\",\n      \"line\": 46",
            "\"id\": \"field:Size.area\",\n      \"line\": 46",
            "\"id\": \"trait:Ord\",\n      \"line\": 53",
            "\"id\": \"fn:largest\",\n      \"line\": 67",
            "\"id\": \"method:Drawing.biggest\",\n      \"line\": 65",
            "\"id\": \"trait:Shape\",\n      \"line\": 15",
            "\"id\": \"type:Circle\",\n      \"line\": 15",
        ] {
            assert!(refs.contains(want), "missing {want}\n{refs}");
        }
    }

    #[test]
    fn doc_comments_and_ids() {
        let src = "\
## The deepest nesting.
MAX_DEPTH := 64

## A colour.
enum Color
    ## The first.
    Red
    Green(level int)

## A point.
struct Point
    ## Across.
    x int
    y int

## Moves it.
fn Point.shift(mut self, dx int)
    self.x += dx

fn main()
    mut p := Point(x: 1, y: 2)
    p.shift(MAX_DEPTH)
    c := Color.Green(level: p.x)
    match c
        Red => print(\"r\")
        Green(n) => print(n)
";
        let out = api_of(src, true);
        for want in [
            "\"id\": \"const:MAX_DEPTH\",\n      \"line\": 2,\n      \"col\": 1,\n      \"doc\": \"The deepest nesting.\",\n      \"signature\": \"MAX_DEPTH := 64\"",
            "\"id\": \"variant:Color.Red\"",
            "\"doc\": \"The first.\"",
            "\"id\": \"field:Color.Green.level\"",
            "\"doc\": \"A point.\"",
            "\"doc\": \"Across.\"",
            "\"doc\": \"Moves it.\"",
            "\"receiver\": \"mut self\"",
            "\"signature\": \"fn Point.shift(mut self, dx int)\"",
        ] {
            assert!(out.contains(want), "missing {want}\n{out}");
        }
        let refs = &out[out.find("\"references\"").unwrap()..];
        for want in [
            "\"id\": \"type:Point\",\n      \"line\": 17,\n      \"col\": 4",
            "\"id\": \"field:Point.x\",\n      \"line\": 18,\n      \"col\": 10",
            "\"id\": \"type:Point\",\n      \"line\": 21",
            "\"id\": \"field:Point.x\",\n      \"line\": 21",
            "\"id\": \"method:Point.shift\",\n      \"line\": 22",
            "\"id\": \"const:MAX_DEPTH\",\n      \"line\": 22",
            "\"id\": \"type:Color\",\n      \"line\": 23",
            "\"id\": \"variant:Color.Green\",\n      \"line\": 23",
            "\"id\": \"field:Color.Green.level\",\n      \"line\": 23",
            "\"id\": \"variant:Color.Red\",\n      \"line\": 25",
            "\"id\": \"variant:Color.Green\",\n      \"line\": 26",
        ] {
            assert!(refs.contains(want), "missing {want}\n{refs}");
        }
        // Undocumented declarations have `null`, not a stray doc.
        assert!(out.contains("\"id\": \"field:Point.y\",\n          \"line\": 14,\n          \"col\": 5,\n          \"doc\": null"));
    }

    #[test]
    fn default_json_is_one_line() {
        let out = outline("t.ail", "## Adds.\nfn add(a int, b int) int\n    a + b\n", false, Form::Json).unwrap();
        assert_eq!(
            out,
            "{\"api\":1,\"file\":\"t.ail\",\"items\":[{\"kind\":\"fn\",\"name\":\"add\",\"id\":\"fn:add\",\"line\":2,\"col\":4,\"doc\":\"Adds.\",\"signature\":\"fn add(a int, b int) int\",\"params\":[{\"name\":\"a\",\"type\":\"int\",\"mut\":false},{\"name\":\"b\",\"type\":\"int\",\"mut\":false}],\"returns\":\"int\",\"effects\":[]}]}\n"
        );
    }

    #[test]
    fn text_outline() {
        let src = "## Adds.\nfn add(a int, b int) int\n    a + b\n\nstruct P\n    x int\n";
        let out = outline("t.ail", src, false, Form::Text).unwrap();
        assert_eq!(out, "    2  fn add(a int, b int) int\n           ## Adds.\n    5  struct P\n    6      x int\n");
    }

    #[test]
    fn rejected_program_has_no_outline() {
        let diags = outline("t.ail", "fn main()\n    x\n", false, Form::Json).unwrap_err();
        assert!(diags.iter().any(|d| !d.warning));
    }

    #[test]
    fn every_example_has_an_outline() {
        let dir = format!("{}/../examples", env!("CARGO_MANIFEST_DIR"));
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|x| x == "ail") {
                let src = std::fs::read_to_string(&path).unwrap();
                let p = path.display().to_string();
                for (symbols, form) in [(false, Form::Json), (true, Form::Pretty), (true, Form::Text)] {
                    if let Err(d) = outline(&p, &src, symbols, form) {
                        panic!("{p}: {d:?}");
                    }
                }
            }
        }
    }
}

/// `self`, `mut self` or `move self` (trait.method.1, own.move.6).
fn receiver_text(r: crate::ast::Receiver) -> &'static str {
    if r.mutable {
        "mut self"
    } else if r.moved {
        "move self"
    } else {
        "self"
    }
}
