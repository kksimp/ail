//! Position queries of `ail api`: `--locals`, `--type` and `--refs`
//! (api.loc.1-3, api.type.1-2, api.refs.1-3, ADRs 0258-0261). The checker
//! records the facts (`check::facts`); this file only selects and prints them.

use std::process::ExitCode;

use super::*;
use crate::check::facts::LocalFact;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Query {
    Locals,
    Type,
    Refs,
}

impl Query {
    pub fn from_flag(a: &str) -> Option<Query> {
        match a {
            "--locals" => Some(Query::Locals),
            "--type" => Some(Query::Type),
            "--refs" => Some(Query::Refs),
            _ => None,
        }
    }
}

/// `ail api --locals|--type|--refs FILE LINE COL [--pretty]`.
pub fn command(q: Query, args: &[&str], json: bool) -> ExitCode {
    let mut pretty = false;
    let mut pos_args = Vec::new();
    for a in args {
        match *a {
            "--pretty" => pretty = true,
            f if Query::from_flag(f).is_some_and(|k| k == q) => {}
            f if !f.starts_with("--") => pos_args.push(f),
            _ => {
                eprint!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    // api.pkg.2: `DIR FILE LINE COL` asks about a file in the context of its whole package.
    let (pkg, path, line, col) = match pos_args[..] {
        [path, line, col] => (None, path, line, col),
        [dir, path, line, col] if std::path::Path::new(dir).is_dir() => (Some(dir), path, line, col),
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let (Ok(line), Ok(col)) = (line.parse::<usize>(), col.parse::<usize>()) else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    match run_pkg(q, pkg, path, &src, line, col) {
        Ok(j) => {
            let mut out = String::new();
            j.write(&mut out, pretty.then_some(0));
            out.push('\n');
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(Failure::Diags(mut diags)) => {
            diag::report(&mut diags, path, &src, json);
            ExitCode::from(2)
        }
        Err(Failure::Nothing(msg)) => {
            // api.query.1: a position with nothing to answer is exit 1.
            let j = Json::Obj(vec![
                ("api", Json::Int(API_VERSION)),
                ("severity", Json::Str("error".into())),
                ("message", Json::Str(msg)),
            ]);
            let mut out = String::new();
            j.write(&mut out, None);
            eprintln!("{out}");
            ExitCode::from(1)
        }
    }
}

pub enum Failure {
    Diags(Vec<diag::Diagnostic>),
    Nothing(String),
}

/// The answer to one query as JSON. A rejected program has no answer (api.out.5).
#[cfg(test)]
pub fn run(q: Query, path: &str, src: &str, line: usize, col: usize) -> Result<Json, Failure> {
    run_pkg(q, None, path, src, line, col)
}

/// `run` for a file inside the package directory `pkg` (api.pkg.2): the
/// package's other root files and modules are loaded too, so `--refs` finds
/// uses in every one of them.
pub fn run_pkg(q: Query, pkg: Option<&str>, path: &str, src: &str, line: usize, col: usize) -> Result<Json, Failure> {
    let (mut prog, mut diags, base) = match pkg {
        None => {
            let (p, d) = crate::modules::load_src(path, src);
            (p, d, 0)
        }
        Some(dir) => crate::modules::load_in(dir, path, src).unwrap_or_else(|e| {
            (Program::default(), vec![diag::Diagnostic::new("api.pkg.2", Pos { line: 1, col: 1 }, e)], 0)
        }),
    };
    let mut resolved = Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check_recording(&prog);
    }
    diags.retain(|d| d.rule != "mod.main.1");
    if diags.iter().any(|d| !d.warning) {
        return Err(Failure::Diags(diags));
    }
    if pkg.is_some() {
        // A package has a file per module: each file's doc comments, by line.
        let mut by_line = std::collections::HashMap::new();
        for f in prog.files.iter().filter(|f| f.path.ends_with(".ail")) {
            let text = if f.base == base { src } else { f.src.as_str() };
            for (l, doc) in docs::by_line(text) {
                by_line.insert(f.base + l, doc);
            }
        }
        docs::attach_lines(&mut prog, &by_line);
    } else {
        docs::attach(&mut prog, src);
    }
    let lines: Vec<&str> = src.lines().collect();
    // `cursor` is in the file; `at` is the same place in the program (api.pkg.2).
    let cursor = Pos { line, col };
    let head = |extra: Vec<(&'static str, Json)>| {
        let mut kvs = vec![
            ("api", Json::Int(API_VERSION)),
            ("file", Json::Str(path.to_string())),
            ("line", Json::Int(line as i64)),
            ("col", Json::Int(col as i64)),
        ];
        kvs.extend(extra);
        Json::Obj(kvs)
    };
    match q {
        Query::Locals => {
            let mut vis = visible_locals(&resolved, &lines, cursor);
            vis.sort_by_key(|l| (l.kind != "self", l.at.map_or((0, 0), |p| (p.line, p.col))));
            Ok(head(vec![("locals", Json::Arr(vis.into_iter().map(local_json).collect()))]))
        }
        Query::Type => {
            let tok = token_at(&lines, cursor).ok_or_else(|| Failure::Nothing(format!("no name or literal at {line}:{col}")))?;
            let start = Pos { line: line + base, col: tok.start };
            let effects = check::effects::effects_of(&prog, &resolved);
            let api = Api::new(&prog, effects);
            let refs = Refs { prog: &prog, resolved: &resolved, found: Vec::new(), generics: Vec::new(), names: Vec::new() };
            let symbol = refs.resolved_at(start, &tok.text);
            // Locals are tracked for the first file only.
            let local = if base == 0 { visible_locals(&resolved, &lines, cursor).into_iter().find(|l| l.name == tok.text) } else { None };
            // The checker's type of the innermost expression starting here.
            let ty = resolved
                .facts
                .exprs
                .iter()
                .find(|(p, _)| *p == start)
                .map(|(_, t)| t.clone())
                .or_else(|| local.as_ref().map(|l| l.ty.clone()))
                .or_else(|| {
                    // The binding site of a local, which is not an expression.
                    resolved.facts.locals.iter().find(|l| l.at == Some(start)).map(|l| l.ty.clone())
                });
            let Some(ty) = ty else {
                return Err(Failure::Nothing(format!("no type recorded for `{}` at {line}:{col}", tok.text)));
            };
            let mut kvs = vec![
                ("text", Json::Str(tok.text.clone())),
                ("start_col", Json::Int(tok.start as i64)),
                ("end_col", Json::Int(tok.end as i64)),
                ("type", Json::Str(ty)),
                ("symbol", Json::opt(symbol.clone())),
            ];
            // api.type.2: a call also says what the callee promises.
            let callee = symbol.as_deref().filter(|id| id.starts_with("fn:") || id.starts_with("method:")).and_then(|id| {
                (0..prog.fns.len()).find(|&i| api.fn_ids[i] == id && !crate::ast::is_prelude(prog.fns[i].pos)).map(|i| api.fn_item(i).json())
            });
            kvs.push(("callee", callee.unwrap_or(Json::Null)));
            Ok(head(kvs))
        }
        Query::Refs => refs_query(&prog, &resolved, &lines, cursor, base, path, head),
    }
}

fn local_json(l: LocalFact) -> Json {
    Json::Obj(vec![
        ("name", Json::Str(l.name)),
        ("type", Json::Str(l.ty)),
        ("mut", Json::Bool(l.mutable)),
        ("kind", Json::Str(l.kind.to_string())),
        ("line", l.at.map_or(Json::Null, |p| Json::Int(p.line as i64))),
        ("col", l.at.map_or(Json::Null, |p| Json::Int(p.col as i64))),
    ])
}

struct Token {
    start: usize,
    end: usize,
    text: String,
}

/// The name or number around (or just before) `at`; columns are characters.
fn token_at(lines: &[&str], at: Pos) -> Option<Token> {
    let chars: Vec<char> = lines.get(at.line.checked_sub(1)?)?.chars().collect();
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut i = at.col.checked_sub(1)?;
    if !chars.get(i).is_some_and(|c| word(*c)) {
        i = i.checked_sub(1).filter(|j| word(chars[*j]))?;
    }
    let (mut a, mut b) = (i, i);
    while a > 0 && word(chars[a - 1]) {
        a -= 1;
    }
    while b + 1 < chars.len() && word(chars[b + 1]) {
        b += 1;
    }
    Some(Token { start: a + 1, end: b + 2, text: chars[a..=b].iter().collect() })
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

/// api.loc.2: a local is in scope from its declaration (after its name on
/// that line) or, for a parameter, loop variable or pattern name, from the
/// first statement of the block it scopes, down to the first line indented
/// less than the line it starts on. Blank and comment lines don't end it.
fn in_scope(l: &LocalFact, lines: &[&str], at: Pos) -> bool {
    let a = l.anchor;
    if a.line > FILE_LINES || at.line < a.line || (at.line == a.line && l.decl.is_some() && at.col <= a.col) {
        return false;
    }
    let Some(first) = lines.get(a.line - 1) else { return false };
    let base = indent_of(first);
    let blank = |s: &str| s.trim().is_empty() || s.trim_start().starts_with('#');
    for n in a.line + 1..=at.line {
        let Some(s) = lines.get(n - 1) else { return false };
        if n == at.line && s.trim().is_empty() {
            // An empty cursor line takes its indent from the cursor.
            if at.col - 1 < base {
                return false;
            }
        } else if !blank(s) && indent_of(s) < base {
            return false;
        }
    }
    true
}

fn visible_locals(resolved: &Resolved, lines: &[&str], at: Pos) -> Vec<LocalFact> {
    resolved.facts.locals.iter().filter(|l| in_scope(l, lines, at)).cloned().collect()
}

/// api.refs.1-3: the symbol under the cursor, then its definition and uses.
fn refs_query(
    prog: &Program,
    resolved: &Resolved,
    lines: &[&str],
    cursor: Pos,
    base: usize,
    path: &str,
    head: impl Fn(Vec<(&'static str, Json)>) -> Json,
) -> Result<Json, Failure> {
    let tok = token_at(lines, cursor).ok_or_else(|| Failure::Nothing(format!("no name at {}:{}", cursor.line, cursor.col)))?;
    let start = Pos { line: cursor.line + base, col: tok.start };
    let effects = check::effects::effects_of(prog, resolved);
    let mut api = Api::new(prog, effects);
    api.whole = true;
    let mut refs = Refs { prog, resolved, found: Vec::new(), generics: Vec::new(), names: Vec::new() };
    refs.program();
    let mut found = refs.found;
    found.retain(|(p, _)| !crate::ast::is_prelude(*p));
    let mut decls = Vec::new();
    flatten(&api.outline_items(), &mut decls);
    // The symbol: a use here, or a declaration here.
    let id = found
        .iter()
        .find(|(p, _)| *p == start)
        .map(|(_, id)| id.clone())
        .or_else(|| decls.iter().find(|(p, _)| *p == start).map(|(_, id)| id.clone()))
        .or_else(|| {
            // api.pkg.3: a use through a module (`geo.origin`) is recorded at the qualifier; the
            // name after the dot asks the same question.
            let chars: Vec<char> = lines.get(cursor.line.checked_sub(1)?)?.chars().collect();
            let dot = tok.start.checked_sub(2).filter(|&i| chars.get(i) == Some(&'.'))?;
            let word = |c: &char| c.is_alphanumeric() || *c == '_' || *c == '.';
            let from = chars[..dot].iter().rposition(|c| !word(c)).map_or(0, |i| i + 1);
            let q = Pos { line: cursor.line + base, col: from + 1 };
            found.iter().find(|(p, _)| *p == q).map(|(_, id)| id.clone())
        });
    let mut at: Vec<(Pos, bool)> = Vec::new();
    let symbol;
    if let Some(id) = id {
        symbol = id.clone();
        at.extend(found.iter().filter(|(_, i)| *i == id).map(|(p, _)| (*p, false)));
        at.extend(decls.iter().filter(|(_, i)| *i == id).map(|(p, _)| (*p, true)));
    } else {
        // A local: its binding and the unresolved names in its scope.
        let scoped = if base == 0 { visible_locals(resolved, lines, cursor) } else { Vec::new() };
        let me = scoped
            .into_iter()
            .find(|l| l.name == tok.text)
            .or_else(|| resolved.facts.locals.iter().find(|l| l.at == Some(start)).cloned());
        let Some(me) = me else {
            return Err(Failure::Nothing(format!("no symbol `{}` at {}:{}", tok.text, cursor.line, cursor.col)));
        };
        symbol = format!("local:{}", me.name);
        if let Some(b) = me.at {
            at.push((b, true));
        }
        for (p, n) in &refs.names {
            if *n == me.name && p.line < FILE_LINES && in_scope(&me, lines, *p) && Some(*p) != me.at {
                at.push((*p, false));
            }
        }
        // `self` has no binding; its uses are all names.
    }
    at.sort_by_key(|(p, def)| (p.line, p.col, !*def));
    at.dedup();
    let out = at
        .into_iter()
        .map(|(p, def)| {
            // Every file knows its own path; in a package the first file is not always the queried one.
            let (file, line) = match prog.file_at(p) {
                Some(f) => (f.path.clone(), p.line - f.base),
                None => (path.to_string(), p.line),
            };
            Json::Obj(vec![
                ("file", Json::Str(file)),
                ("line", Json::Int(line as i64)),
                ("col", Json::Int(p.col as i64)),
                ("def", Json::Bool(def)),
            ])
        })
        .collect();
    Ok(head(vec![("symbol", Json::Str(symbol)), ("refs", Json::Arr(out))]))
}

fn flatten(items: &[Item], out: &mut Vec<(Pos, String)>) {
    for i in items {
        out.push((i.pos, i.id.clone()));
        for (_, ms) in &i.members {
            flatten(ms, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "\
## A point.
struct Point
    x int
    y int

fn Point.shift(mut self, dx int)
    requires dx >= 0
    self.x += dx

fn total(items List[Point]) int
    mut sum := 0
    for p in items
        d := p.x + 1
        sum += d
    # after loop
    sum

fn main()
    pts := [Point(x: 1, y: 2)]
    mut q := pts[0]
    q.shift(3)
    n := total(pts)
    print(\"{n} {q.x}\")
";

    fn ask(q: Query, line: usize, col: usize) -> String {
        match run(q, "t.ail", SRC, line, col) {
            Ok(j) => {
                let mut out = String::new();
                j.write(&mut out, None);
                out
            }
            Err(Failure::Nothing(m)) => format!("nothing: {m}"),
            Err(Failure::Diags(d)) => panic!("{d:?}"),
        }
    }

    #[test]
    fn locals_in_a_loop_body() {
        let out = ask(Query::Locals, 14, 9);
        let want = "\"locals\":[\
{\"name\":\"items\",\"type\":\"List[Point]\",\"mut\":false,\"kind\":\"param\",\"line\":10,\"col\":10},\
{\"name\":\"sum\",\"type\":\"int\",\"mut\":true,\"kind\":\"binding\",\"line\":11,\"col\":9},\
{\"name\":\"p\",\"type\":\"Point\",\"mut\":false,\"kind\":\"loop\",\"line\":12,\"col\":9},\
{\"name\":\"d\",\"type\":\"int\",\"mut\":false,\"kind\":\"binding\",\"line\":13,\"col\":9}]}";
        assert!(out.ends_with(want), "{out}");
    }

    #[test]
    fn locals_end_with_their_block() {
        // api.loc.2: after the loop the loop variable and `d` are gone.
        let out = ask(Query::Locals, 16, 5);
        assert!(out.contains("\"name\":\"sum\""), "{out}");
        assert!(!out.contains("\"name\":\"p\"") && !out.contains("\"name\":\"d\""), "{out}");
        // A local isn't in scope on its own line, or before it.
        assert!(!ask(Query::Locals, 22, 5).contains("\"name\":\"n\""));
        assert!(!ask(Query::Locals, 19, 5).contains("\"name\":\"pts\""));
    }

    #[test]
    fn locals_include_self() {
        let out = ask(Query::Locals, 8, 9);
        assert!(out.contains("{\"name\":\"self\",\"type\":\"Point\",\"mut\":true,\"kind\":\"self\",\"line\":null,\"col\":null}"), "{out}");
        assert!(out.contains("{\"name\":\"dx\",\"type\":\"int\""), "{out}");
    }

    #[test]
    fn type_of_a_name_a_call_and_a_method() {
        assert!(ask(Query::Type, 20, 14).contains("\"text\":\"pts\",\"start_col\":14,\"end_col\":17,\"type\":\"List[Point]\",\"symbol\":null,\"callee\":null"));
        let call = ask(Query::Type, 22, 11);
        assert!(call.contains("\"type\":\"int\",\"symbol\":\"fn:total\",\"callee\":{\"kind\":\"fn\""), "{call}");
        assert!(call.contains("\"signature\":\"fn total(items List[Point]) int\""), "{call}");
        let method = ask(Query::Type, 21, 8);
        assert!(method.contains("\"symbol\":\"method:Point.shift\""), "{method}");
        assert!(method.contains("\"requires\":[\"dx >= 0\"]"), "{method}");
    }

    #[test]
    fn type_of_a_binding_site_and_a_field() {
        assert!(ask(Query::Type, 11, 9).contains("\"text\":\"sum\",\"start_col\":9,\"end_col\":12,\"type\":\"int\""));
        assert!(ask(Query::Type, 13, 16).contains("\"text\":\"x\""));
        assert!(ask(Query::Type, 13, 16).contains("\"type\":\"int\""));
    }

    #[test]
    fn type_with_nothing_there() {
        assert!(ask(Query::Type, 4, 1).starts_with("nothing:"), "{}", ask(Query::Type, 4, 1));
    }

    #[test]
    fn refs_of_a_type_and_a_field() {
        let out = ask(Query::Refs, 2, 9);
        assert!(out.contains("\"symbol\":\"type:Point\""), "{out}");
        assert!(out.contains("{\"file\":\"t.ail\",\"line\":2,\"col\":8,\"def\":true}"), "{out}");
        assert!(out.contains("{\"file\":\"t.ail\",\"line\":19,\"col\":13,\"def\":false}"), "{out}");
        let field = ask(Query::Refs, 8, 10);
        assert!(field.contains("\"symbol\":\"field:Point.x\""), "{field}");
        assert!(field.matches("\"def\":false").count() == 4, "{field}");
    }

    #[test]
    fn refs_of_a_local_stay_in_its_scope() {
        let out = ask(Query::Refs, 14, 9);
        assert!(out.contains("\"symbol\":\"local:sum\""), "{out}");
        assert!(out.ends_with("\"refs\":[{\"file\":\"t.ail\",\"line\":11,\"col\":9,\"def\":true},{\"file\":\"t.ail\",\"line\":14,\"col\":9,\"def\":false},{\"file\":\"t.ail\",\"line\":16,\"col\":5,\"def\":false}]}"), "{out}");
    }

    #[test]
    fn struct_and_variant_fields_have_types() {
        // api.out.3: fields, and the fields of a variant, carry `type`.
        let out = outline("t.ail", "enum Shape\n    Rect(w int, h int)\n    Dot\n", false, Form::Json).unwrap();
        assert!(out.contains("\"id\":\"field:Shape.Rect.w\""), "{out}");
        assert!(out.contains("\"signature\":\"w int\",\"type\":\"int\""), "{out}");
    }
}
