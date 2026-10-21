//! `ail edit PATH [--dry-run] [--diff] [--if-clean] [--test] [--json] < EDITS`:
//! structured edits by declaration id (edit.op.1-7, ADRs 0397, 0399). EDITS
//! are compact hunks under declaration names (edit.in.3) or a JSON list of
//! operations; they apply in order to the file or package at
//! PATH, every changed file is formatted, the program is checked, and the
//! answer is the diagnostics of the new state. Cheaper and safer than
//! rewriting a file: the agent names what to change, not where.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::diag::json_str;

pub const USAGE: &str = "usage: ail edit PATH [--dry-run] [--diff] [--if-clean] [--test] [--json] < EDITS\n\
EDITS are hunks: an id line, then lines starting with `-` (remove), `+` (add) or a space (keep):\n\
  average                                 one `-` and one `+` line: a whole line, or else a unique part of one\n\
  -total / xs.len()                       `+` lines are indented like the line they replace\n\
  +if xs.len() == 0 { 0 } else { total / xs.len() }\n\
  rename OLD NEW                          the declaration and every reference\n\
or a JSON list; each item has an \"op\":\n\
  {\"op\":\"replace_body\",\"id\":\"fn:f\",\"body\":\"x + 1\"}       the lines after the signature (contracts stay)\n\
  {\"op\":\"replace\",\"id\":\"type:P\",\"text\":\"struct P\\n    x int\"} the whole declaration, doc comment included\n\
  {\"op\":\"insert\",\"text\":\"fn g() int\\n    1\",\"after\":\"fn:f\"}  or \"before\":ID; neither: the end of the file\n\
  {\"op\":\"delete\",\"id\":\"fn:old\"}\n\
  {\"op\":\"rename\",\"id\":\"fn:f\",\"to\":\"g\"}                  the declaration and every reference\n\
  {\"op\":\"import\",\"module\":\"std.fs\"}                    adds `use std.fs` if missing\n\
In a package, ids may carry the module (fn:geo.origin) and \"file\" picks the file for insert and import.\n\
A bare name (average, P.m) stands for the one declaration with that name. --test runs `ail test PATH` after a clean edit.\n";

/// A JSON value from the edit list (edit.in.1).
#[derive(Debug, Clone, PartialEq)]
enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

impl J {
    fn get(&self, key: &str) -> Option<&J> {
        match self {
            J::Obj(kv) => kv.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn str(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }
}

fn parse_json(s: &str) -> Result<J, String> {
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    let v = value(&b, &mut i)?;
    skip_ws(&b, &mut i);
    if i != b.len() {
        return Err(format!("unexpected text after the JSON value at character {i}"));
    }
    Ok(v)
}

fn skip_ws(b: &[char], i: &mut usize) {
    while *i < b.len() && b[*i].is_whitespace() {
        *i += 1;
    }
}

fn value(b: &[char], i: &mut usize) -> Result<J, String> {
    skip_ws(b, i);
    let at = *i;
    match b.get(*i) {
        None => Err("the JSON ends early".into()),
        Some('{') => {
            *i += 1;
            let mut kv = Vec::new();
            skip_ws(b, i);
            if b.get(*i) == Some(&'}') {
                *i += 1;
                return Ok(J::Obj(kv));
            }
            loop {
                skip_ws(b, i);
                let J::Str(k) = value(b, i)? else { return Err(format!("expected a key at character {}", *i)) };
                skip_ws(b, i);
                if b.get(*i) != Some(&':') {
                    return Err(format!("expected `:` at character {}", *i));
                }
                *i += 1;
                kv.push((k, value(b, i)?));
                skip_ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some('}') => {
                        *i += 1;
                        return Ok(J::Obj(kv));
                    }
                    _ => return Err(format!("expected `,` or `}}` at character {}", *i)),
                }
            }
        }
        Some('[') => {
            *i += 1;
            let mut xs = Vec::new();
            skip_ws(b, i);
            if b.get(*i) == Some(&']') {
                *i += 1;
                return Ok(J::Arr(xs));
            }
            loop {
                xs.push(value(b, i)?);
                skip_ws(b, i);
                match b.get(*i) {
                    Some(',') => *i += 1,
                    Some(']') => {
                        *i += 1;
                        return Ok(J::Arr(xs));
                    }
                    _ => return Err(format!("expected `,` or `]` at character {}", *i)),
                }
            }
        }
        Some('"') => {
            *i += 1;
            let mut s = String::new();
            while let Some(&c) = b.get(*i) {
                *i += 1;
                match c {
                    '"' => return Ok(J::Str(s)),
                    '\\' => {
                        let e = *b.get(*i).ok_or("the JSON ends inside a string")?;
                        *i += 1;
                        match e {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            'b' => s.push('\u{8}'),
                            'f' => s.push('\u{c}'),
                            'u' => {
                                let hex: String = b.get(*i..*i + 4).ok_or("bad \\u escape")?.iter().collect();
                                *i += 4;
                                let n = u32::from_str_radix(&hex, 16).map_err(|_| "bad \\u escape")?;
                                s.push(char::from_u32(n).unwrap_or('\u{fffd}'));
                            }
                            c => s.push(c),
                        }
                    }
                    c => s.push(c),
                }
            }
            Err("the JSON ends inside a string".into())
        }
        Some(c) if *c == '-' || c.is_ascii_digit() => {
            while *i < b.len() && (b[*i].is_ascii_digit() || "+-.eE".contains(b[*i])) {
                *i += 1;
            }
            let t: String = b[at..*i].iter().collect();
            t.parse().map(J::Num).map_err(|_| format!("bad number `{t}`"))
        }
        _ => {
            for (word, v) in [("true", J::Bool(true)), ("false", J::Bool(false)), ("null", J::Null)] {
                if b[at..].iter().take(word.len()).copied().eq(word.chars()) {
                    *i += word.len();
                    return Ok(v);
                }
            }
            Err(format!("unexpected `{}` at character {at}", b[at]))
        }
    }
}

/// A top-level declaration found in a file's text (edit.id.1).
#[derive(Debug, Clone)]
struct Decl {
    id: String,
    /// The bare name a rename changes.
    name: String,
    /// 0-based: first line (doc comment and attributes included), the
    /// declaration's own line, the last line of its signature, and the line
    /// after its last.
    start: usize,
    head: usize,
    sig_end: usize,
    end: usize,
}

/// The id and name a column-0 line declares, if any (edit.id.1).
fn declares(line: &str) -> Option<(String, String)> {
    let mut rest = line;
    while let Some(r) = rest.strip_prefix('@') {
        // `@test fn ...`: an attribute before the declaration on its line.
        rest = r.split_once(' ').map(|(_, t)| t.trim_start())?;
    }
    for w in ["pub ", "unsafe "] {
        if let Some(r) = rest.strip_prefix(w) {
            rest = r;
        }
    }
    let word = |s: &str| -> String { s.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect() };
    if let Some(r) = rest.strip_prefix("fn ") {
        let first = word(r);
        let after = &r[first.len()..];
        if let Some(m) = after.strip_prefix('.') {
            let m = word(m);
            return Some((format!("method:{first}.{m}"), m));
        }
        return Some((format!("fn:{first}"), first));
    }
    for w in ["error ", "unique "] {
        if let Some(r) = rest.strip_prefix(w) {
            rest = r;
        }
    }
    for (kw, kind) in [("struct ", "type"), ("enum ", "type"), ("trait ", "trait")] {
        if let Some(r) = rest.strip_prefix(kw) {
            let n = word(r);
            return Some((format!("{kind}:{n}"), n));
        }
    }
    let n = word(rest);
    if !n.is_empty() && rest[n.len()..].trim_start().starts_with(":=") {
        return Some((format!("const:{n}"), n));
    }
    None
}

/// The declarations of `src`, in order.
fn decls(src: &str) -> Vec<Decl> {
    let lines: Vec<&str> = src.split('\n').collect();
    let mut found: Vec<Decl> = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        if l.starts_with([' ', '#', '\t']) || l.trim().is_empty() {
            continue;
        }
        // A column-0 line ends the previous declaration either way.
        let Some((id, name)) = declares(l) else { continue };
        let mut start = i;
        while start > 0 && (lines[start - 1].starts_with("##") || lines[start - 1].starts_with('@')) {
            start -= 1;
        }
        found.push(Decl { id, name, start, head: i, sig_end: signature_end(&lines, i), end: i + 1 });
    }
    // Each runs to the next column-0 code line, without trailing blank and comment lines.
    for k in 0..found.len() {
        let mut j = found[k].sig_end + 1;
        while j < lines.len() {
            let l = lines[j];
            if !(l.starts_with([' ', '\t']) || l.trim().is_empty() || (l.starts_with('#') && !l.starts_with("##"))) {
                break;
            }
            j += 1;
        }
        while j > found[k].sig_end + 1 && {
            let t = lines[j - 1].trim();
            t.is_empty() || (t.starts_with('#') && !lines[j - 1].starts_with(' '))
        } {
            j -= 1;
        }
        found[k].end = j;
    }
    found
}

/// The last line of the signature that starts on line `i`: where its
/// brackets close (a long parameter list spans lines).
fn signature_end(lines: &[&str], i: usize) -> usize {
    let mut depth: i32 = 0;
    let mut j = i;
    while j < lines.len() {
        let mut in_str = false;
        let mut prev = ' ';
        for c in lines[j].chars() {
            match c {
                '"' if prev != '\\' => in_str = !in_str,
                '#' if !in_str => break,
                '(' | '[' | '{' if !in_str => depth += 1,
                ')' | ']' | '}' if !in_str => depth -= 1,
                _ => {}
            }
            prev = c;
        }
        if depth <= 0 {
            return j;
        }
        j += 1;
    }
    i
}

/// The files of the program at `path`: the file, or every `.ail` file of a package.
fn files_of(path: &str) -> Vec<PathBuf> {
    let p = Path::new(path);
    if !p.is_dir() {
        return vec![p.to_path_buf()];
    }
    let mut out = Vec::new();
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for e in entries {
            let name = e.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if e.is_dir() && !name.starts_with('.') && name != "target" {
                walk(&e, out);
            } else if name.ends_with(".ail") {
                out.push(e);
            }
        }
    }
    walk(p, &mut out);
    out
}

/// The module path of `file` inside package `root`: `geo` for `root/geo/a.ail`.
fn module_of(root: &Path, file: &Path) -> String {
    let rel = file.parent().and_then(|d| d.strip_prefix(root).ok()).map(|d| d.to_path_buf()).unwrap_or_default();
    rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect::<Vec<_>>().join(".")
}

/// `fn:origin` in module `geo` is `fn:geo.origin` (api.pkg.1).
fn qualified(id: &str, module: &str) -> String {
    match id.split_once(':') {
        Some((kind, rest)) if !module.is_empty() => format!("{kind}:{module}.{rest}"),
        _ => id.to_string(),
    }
}

/// The working copy: each file's text, keyed by path.
struct Work {
    root: PathBuf,
    package: bool,
    files: BTreeMap<PathBuf, String>,
}

impl Work {
    /// The file and declaration an id names (edit.id.1-2).
    fn find(&self, id: &str) -> Result<(PathBuf, Decl), String> {
        let mut hits = Vec::new();
        for (f, src) in &self.files {
            let module = self.module(f);
            for d in decls(src) {
                // edit.id.3: a bare name stands for `kind:name`.
                let bare = |d_id: &str| !id.contains(':') && d_id.split_once(':').is_some_and(|(_, n)| n == id);
                if d.id == id
                    || bare(&d.id)
                    || (!module.is_empty() && (qualified(&d.id, &module) == id || bare(&qualified(&d.id, &module))))
                {
                    hits.push((f.clone(), d));
                }
            }
        }
        match hits.len() {
            1 => Ok(hits.pop().expect("one hit")),
            0 => Err(format!("no declaration has the id `{id}` (edit.id.1)")),
            _ => Err(format!(
                "`{id}` names {} declarations; qualify it with its module, e.g. `fn:geo.origin` (edit.id.2)",
                hits.len()
            )),
        }
    }

    /// The module of `file`: empty for a single file and a package's root.
    fn module(&self, file: &Path) -> String {
        if self.package { module_of(&self.root, file) } else { String::new() }
    }

    /// The file an insert or import goes to (edit.op.3, edit.op.6).
    fn target(&self, op: &J) -> Result<PathBuf, String> {
        if let Some(f) = op.str("file") {
            let p = if self.package { self.root.join(f) } else { PathBuf::from(f) };
            return self
                .files
                .contains_key(&p)
                .then_some(p)
                .ok_or(format!("`{f}` is not a file of the program (edit.op.3)"));
        }
        if !self.package {
            return Ok(self.root.clone());
        }
        // A package's root module: its first file at the top.
        self.files
            .keys()
            .find(|f| f.parent() == Some(self.root.as_path()))
            .cloned()
            .ok_or_else(|| "say which file with \"file\" (edit.op.3)".to_string())
    }
}

/// `text` without its common indentation, indented by `by` spaces.
fn reindent(text: &str, by: usize) -> Vec<String> {
    let lines: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
    let common = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() { String::new() } else { format!("{}{}", " ".repeat(by), &l[common.min(l.len())..]) }
        })
        .collect()
}

fn splice(src: &str, from: usize, to: usize, new: &[String]) -> String {
    let mut lines: Vec<String> = src.split('\n').map(str::to_string).collect();
    let to = to.min(lines.len());
    lines.splice(from..to, new.iter().cloned());
    lines.join("\n")
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// Apply a hunk (edit.op.7) to lines `from..to` of `src`: the `-` and context
/// lines are found by their trimmed text, and `+` lines are indented like the
/// first line found. One `-` and one `+` line that match no whole line
/// replace a unique part of one.
fn hunk(src: &str, from: usize, to: usize, lines: &[&str]) -> Result<String, String> {
    let file: Vec<&str> = src.split('\n').collect();
    let scope = &file[from..to.min(file.len())];
    let text = |l: &str| l.get(1..).unwrap_or("").to_string();
    let minus: Vec<&&str> = lines.iter().filter(|l| l.starts_with('-')).collect();
    let plus: Vec<&&str> = lines.iter().filter(|l| l.starts_with('+')).collect();
    let old: Vec<String> = lines.iter().filter(|l| !l.starts_with('+')).map(|l| text(l).trim().to_string()).collect();
    if old.is_empty() {
        return Err("a hunk needs a `-` or context line to find its place".into());
    }
    if let ([m], [p]) = (minus.as_slice(), plus.as_slice())
        && lines.len() == 2
        && !scope.iter().any(|l| l.trim() == old[0])
    {
        let (part, new) = (text(m).trim().to_string(), text(p).trim().to_string());
        let hits: Vec<(usize, usize)> = scope
            .iter()
            .enumerate()
            .flat_map(|(i, l)| l.match_indices(part.as_str()).map(move |(at, _)| (i, at)))
            .collect();
        return match hits.as_slice() {
            [(i, at)] => {
                let line = scope[*i];
                let changed = format!("{}{new}{}", &line[..*at], &line[at + part.len()..]);
                Ok(splice(src, from + i, from + i + 1, &[changed]))
            }
            [] => Err(format!("`{part}` is not found")),
            _ => Err(format!("`{part}` occurs {} times; quote more of its line", hits.len())),
        };
    }
    let at: Vec<usize> = (0..=scope.len().saturating_sub(old.len()))
        .filter(|&i| scope.len() >= old.len() && scope[i..i + old.len()].iter().zip(&old).all(|(l, o)| l.trim() == o))
        .collect();
    let i = match at.as_slice() {
        [i] => *i,
        [] => return Err(format!("the lines `{}` are not found", old.join("` / `"))),
        _ => return Err(format!("the `-` and context lines match {} places; add a context line", at.len())),
    };
    let first_old = lines.iter().find(|l| !l.starts_with('+')).map(|l| text(l)).unwrap_or_default();
    let shift = indent_of(scope[i]) as i64 - indent_of(&first_old) as i64;
    let mut new = Vec::new();
    let mut k = i;
    for l in lines {
        let t = text(l);
        match l.as_bytes().first() {
            Some(b'+') if t.trim().is_empty() => new.push(String::new()),
            Some(b'+') => {
                let ind = (indent_of(&t) as i64 + shift).max(0) as usize;
                new.push(format!("{}{}", " ".repeat(ind), t.trim_start()));
            }
            Some(b'-') => k += 1,
            _ => {
                new.push(scope[k].to_string());
                k += 1;
            }
        }
    }
    Ok(splice(src, from + i, from + k, &new))
}

/// The compact form of the edits (edit.in.3): hunks under an id line, and
/// `rename OLD NEW` lines.
fn parse_compact(input: &str) -> Result<Vec<J>, String> {
    let mut ops = Vec::new();
    let mut cur: Option<(Option<String>, Vec<String>)> = None;
    let flush = |cur: Option<(Option<String>, Vec<String>)>, ops: &mut Vec<J>| -> Result<(), String> {
        let Some((id, lines)) = cur else { return Ok(()) };
        if lines.is_empty() {
            return Err(format!("`{}` has no `-`, `+` or context lines (edit.in.3)", id.unwrap_or_default()));
        }
        let mut kv = vec![("op".to_string(), J::Str("hunk".into()))];
        if let Some(id) = id {
            kv.push(("id".into(), J::Str(id)));
        }
        kv.push(("lines".into(), J::Arr(lines.into_iter().map(J::Str).collect())));
        ops.push(J::Obj(kv));
        Ok(())
    };
    for (n, line) in input.lines().enumerate() {
        match line.as_bytes().first() {
            None => {}
            Some(b'-' | b'+' | b' ') => cur.get_or_insert((None, Vec::new())).1.push(line.to_string()),
            _ => {
                flush(cur.take(), &mut ops)?;
                match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                    ["rename", id, to] => ops.push(J::Obj(vec![
                        ("op".into(), J::Str("rename".into())),
                        ("id".into(), J::Str(id.to_string())),
                        ("to".into(), J::Str(to.to_string())),
                    ])),
                    [id] => cur = Some((Some(id.trim_end_matches(':').to_string()), Vec::new())),
                    _ => {
                        return Err(format!(
                            "line {}: expected an id, `rename OLD NEW`, or a line starting with `-`, `+` or a space (edit.in.3)",
                            n + 1
                        ));
                    }
                }
            }
        }
    }
    flush(cur, &mut ops)?;
    if ops.is_empty() {
        return Err("no edits (edit.in.3)".into());
    }
    Ok(ops)
}

/// Apply one operation (edit.op.1-7).
fn apply(work: &mut Work, op: &J, path: &str) -> Result<(), String> {
    let kind = op.str("op").ok_or("each edit needs an \"op\" (edit.in.1)")?;
    let need = |k: &str| op.str(k).ok_or(format!("`{kind}` needs \"{k}\" (edit.in.1)"));
    match kind {
        // edit.op.1: the lines after the signature; leading `requires`/`ensures` stay unless the new body has its own.
        "replace_body" => {
            let (f, d) = work.find(need("id")?)?;
            let body = need("body")?;
            let src = &work.files[&f];
            let lines: Vec<&str> = src.split('\n').collect();
            let mut keep = d.sig_end + 1;
            let new_has =
                body.lines().any(|l| matches!(l.trim_start().split(' ').next(), Some("requires" | "ensures")));
            if !new_has {
                while keep < d.end && matches!(lines[keep].trim_start().split(' ').next(), Some("requires" | "ensures"))
                {
                    keep += 1;
                }
            }
            let new = splice(src, keep, d.end, &reindent(body, 4));
            work.files.insert(f, new);
        }
        // edit.op.2: the whole declaration, from its doc comment to its last line.
        "replace" => {
            let (f, d) = work.find(need("id")?)?;
            let new = splice(&work.files[&f], d.start, d.end, &reindent(need("text")?, 0));
            work.files.insert(f, new);
        }
        // edit.op.3: a new declaration, after or before another, or at the end.
        "insert" => {
            let text = reindent(need("text")?, 0);
            let (f, at, pad_before) = if let Some(id) = op.str("after") {
                let (f, d) = work.find(id)?;
                (f, d.end, true)
            } else if let Some(id) = op.str("before") {
                let (f, d) = work.find(id)?;
                (f, d.start, false)
            } else {
                let f = work.target(op)?;
                let n = work.files[&f].trim_end_matches('\n').split('\n').count();
                (f, n, true)
            };
            let mut block = Vec::new();
            if pad_before {
                block.push(String::new());
            }
            block.extend(text);
            if !pad_before {
                block.push(String::new());
            }
            let new = splice(&work.files[&f], at, at, &block);
            work.files.insert(f, new);
        }
        // edit.op.4
        "delete" => {
            let (f, d) = work.find(need("id")?)?;
            let new = splice(&work.files[&f], d.start, d.end, &[]);
            work.files.insert(f, new);
        }
        // edit.op.7: lines found by their text inside one declaration (or the file), then replaced.
        "hunk" => {
            let lines: Vec<&str> = match op.get("lines") {
                Some(J::Arr(xs)) => xs.iter().filter_map(|x| x.as_str()).collect(),
                _ => return Err("`hunk` needs \"lines\" (edit.in.1)".into()),
            };
            let (f, from, to, place) = match op.str("id") {
                Some(id) => {
                    let (f, d) = work.find(id)?;
                    (f, d.start, d.end, format!("`{}`", d.id))
                }
                None => {
                    let f = work.target(op)?;
                    let n = work.files[&f].split('\n').count();
                    let place = f.display().to_string();
                    (f, 0, n, place)
                }
            };
            let new = hunk(&work.files[&f], from, to, &lines).map_err(|e| format!("{e} in {place} (edit.op.7)"))?;
            work.files.insert(f, new);
        }
        // edit.op.5: through the references of `ail api --symbols`.
        "rename" => rename(work, need("id")?, need("to")?, path)?,
        // edit.op.6
        "import" => {
            let module = need("module")?;
            let f = work.target(op)?;
            let src = &work.files[&f];
            let line = format!("use {module}");
            let lines: Vec<&str> = src.split('\n').collect();
            if lines.iter().any(|l| l.trim_end() == line) {
                return Ok(());
            }
            let at = match lines.iter().rposition(|l| l.starts_with("use ")) {
                Some(i) => i + 1,
                None => lines.iter().position(|l| !l.starts_with('#')).unwrap_or(0),
            };
            let mut block = vec![line];
            if !lines.iter().any(|l| l.starts_with("use ")) && lines.get(at).is_some_and(|l| !l.trim().is_empty()) {
                block.push(String::new());
            }
            let new = splice(src, at, at, &block);
            work.files.insert(f, new);
        }
        other => {
            return Err(format!(
                "unknown op `{other}`; the ops are replace_body, replace, insert, delete, rename and import (edit.in.1)"
            ));
        }
    }
    Ok(())
}

/// edit.op.5: rename the declaration `id` and every reference to it. The
/// program as edited so far must check, since references come from the checker.
fn rename(work: &mut Work, id: &str, to: &str, path: &str) -> Result<(), String> {
    if to.is_empty()
        || !to.chars().all(|c| c.is_alphanumeric() || c == '_')
        || to.starts_with(|c: char| c.is_ascii_digit())
    {
        return Err(format!("`{to}` is not a name (edit.op.5)"));
    }
    // edit.id.3: a bare name becomes the id it stands for.
    let full;
    let id = if id.contains(':') {
        id
    } else {
        full = work.find(id)?.1.id;
        full.as_str()
    };
    let (df, line, col, old) = declared_at(work, id)?;
    // Write the working copy where the loader reads it, then put the files back.
    let saved = save(work)?;
    let result = (|| {
        let (prog, diags) = crate::modules::load(path)?;
        let (diags, resolved) = if diags.is_empty() { crate::check::check(&prog) } else { (diags, Default::default()) };
        // As in `check_quietly`: a single file needs no `main`.
        let single = !Path::new(path).is_dir();
        if let Some(d) = diags.iter().find(|d| !d.warning && !(single && d.rule == "mod.main.1")) {
            return Err(format!(
                "a rename needs a program that checks; first error: {} [{}] (edit.op.5)",
                d.message, d.rule
            ));
        }
        Ok(crate::api::references(&prog, &resolved, Path::new(path).is_dir()))
    })();
    restore(saved);
    let refs = result?;
    // References name a package's declarations with their module (api.pkg.1).
    let bare = id.split_once(':').map_or(id, |(_, r)| r);
    let kind = id.split_once(':').map_or("", |(k, _)| k);
    let want = if work.package && !work.module(&df).is_empty() && !bare.starts_with(&format!("{}.", work.module(&df))) {
        qualified(id, &work.module(&df))
    } else {
        format!("{kind}:{bare}")
    };
    let mut by_file: BTreeMap<PathBuf, Vec<(usize, usize)>> = BTreeMap::new();
    for (file, line, col, rid) in refs {
        if rid == want {
            let f = if work.package { PathBuf::from(file) } else { work.root.clone() };
            by_file.entry(f).or_default().push((line, col));
        }
    }
    // The declaration's own name.
    by_file.entry(df).or_default().push((line, col));
    for (f, mut spots) in by_file {
        let Some(src) = work.files.get(&f) else { continue };
        spots.sort_unstable();
        spots.dedup();
        let mut lines: Vec<String> = src.split('\n').map(str::to_string).collect();
        for &(line, col) in spots.iter().rev() {
            let Some(l) = lines.get_mut(line - 1) else { continue };
            // A qualified reference (`geo.origin`) points at its qualifier.
            let at: Option<usize> = l.char_indices().nth(col - 1).map(|(i, _)| i).map(|mut i| {
                while !l[i..].starts_with(&*old) {
                    let word = l[i..].find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(l.len() - i);
                    if word == 0 || !l[i + word..].starts_with('.') {
                        break;
                    }
                    i += word + 1;
                }
                i
            });
            match at {
                Some(i) if l[i..].starts_with(&old) => l.replace_range(i..i + old.len(), to),
                _ => {
                    return Err(format!(
                        "{}:{line}:{col} doesn't hold `{old}`; nothing was renamed (edit.op.5)",
                        f.display()
                    ));
                }
            }
        }
        work.files.insert(f, lines.join("\n"));
    }
    Ok(())
}

/// Where `id` is declared: its file, 1-based line and column, and its name.
/// Fields and variants are found inside their type (edit.op.5).
fn declared_at(work: &Work, id: &str) -> Result<(PathBuf, usize, usize, String), String> {
    if let Some((kind @ ("field" | "variant"), rest)) = id.split_once(':') {
        let (owner, member) = rest.rsplit_once('.').ok_or(format!("`{id}` names no member (edit.id.1)"))?;
        let (f, d) = work.find(&format!("type:{owner}"))?;
        let lines: Vec<&str> = work.files[&f].split('\n').collect();
        for (i, l) in lines.iter().enumerate().take(d.end).skip(d.head + 1) {
            let t = l.trim_start();
            let word: String = t.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if word == member && l.len() - t.len() == 4 {
                return Ok((f, i + 1, 5, member.to_string()));
            }
        }
        return Err(format!("no {kind} `{member}` in `{owner}` (edit.id.1)"));
    }
    let (f, d) = work.find(id)?;
    let head = work.files[&f].split('\n').nth(d.head).unwrap_or("").to_string();
    let col = name_column(&head, &d).ok_or(format!("can't find `{}` on its declaration line (edit.op.5)", d.name))?;
    Ok((f, d.head + 1, col, d.name))
}

/// The 1-based column of a declaration's name on its line.
fn name_column(head: &str, d: &Decl) -> Option<usize> {
    let kw = ["fn ", "struct ", "enum ", "trait "];
    let mut from = 0;
    for k in kw {
        if let Some(i) = head.find(k) {
            from = i + k.len();
            break;
        }
    }
    let rest = &head[from..];
    let off = if d.id.starts_with("method:") { rest.find('.')? + 1 } else { 0 };
    let i = from + off;
    head[i..].starts_with(&d.name).then(|| head[..i].chars().count() + 1)
}

type Saved = Vec<(PathBuf, Option<String>)>;

/// Write the working copy, returning what was there before.
fn save(work: &Work) -> Result<Saved, String> {
    let mut saved = Vec::new();
    for (f, src) in &work.files {
        let before = std::fs::read_to_string(f).ok();
        if before.as_deref() != Some(src.as_str()) {
            std::fs::write(f, src).map_err(|e| format!("can't write {}: {e}", f.display()))?;
            saved.push((f.clone(), before));
        }
    }
    Ok(saved)
}

fn restore(saved: Saved) {
    for (f, before) in saved {
        match before {
            Some(s) => {
                let _ = std::fs::write(&f, s);
            }
            None => {
                let _ = std::fs::remove_file(&f);
            }
        }
    }
}

pub fn command(args: &[&str], json: bool) -> ExitCode {
    // edit.out.3: `--dry-run` writes nothing and prints the diff; `--diff` prints it and writes.
    let (mut dry, mut show_diff, mut if_clean, mut test) = (false, false, false, false);
    let mut paths = Vec::new();
    for a in args {
        match *a {
            "--dry-run" => dry = true,
            "--diff" => show_diff = true,
            "--if-clean" => if_clean = true,
            "--test" => test = true,
            a if a.starts_with("--") => {
                eprint!("error: unknown option `{a}`\n{USAGE}");
                return ExitCode::from(2);
            }
            a => paths.push(a),
        }
    }
    let [path] = paths.as_slice() else {
        eprint!("{USAGE}");
        return ExitCode::from(2);
    };
    let mut input = String::new();
    if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input) {
        eprintln!("error: can't read the edits from standard input: {e}");
        return ExitCode::from(2);
    }
    // edit.in.3: input that doesn't start with `[` or `{` is the compact form.
    let compact = !matches!(input.trim_start().as_bytes().first(), Some(b'[' | b'{'));
    let parsed = if compact {
        match parse_compact(&input) {
            Ok(ops) => Ok(J::Arr(ops)),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        parse_json(&input)
    };
    let ops = match parsed {
        Ok(J::Arr(xs)) => xs,
        Ok(o @ J::Obj(_)) => vec![o],
        Ok(_) => {
            eprintln!("error: the edits are a JSON list of objects (edit.in.1)");
            return ExitCode::from(2);
        }
        Err(e) => {
            eprintln!("error: the edits aren't JSON: {e} (edit.in.1)");
            return ExitCode::from(2);
        }
    };
    let root = PathBuf::from(path);
    let package = root.is_dir();
    let mut files = BTreeMap::new();
    for f in files_of(path) {
        match std::fs::read_to_string(&f) {
            Ok(s) => {
                files.insert(f, s);
            }
            Err(e) => {
                eprintln!("error: can't read {}: {e}", f.display());
                return ExitCode::from(2);
            }
        }
    }
    let original = files.clone();
    let mut work = Work { root, package, files };
    // edit.in.2: all or nothing; the first failing edit stops the run and nothing is written.
    for (k, op) in ops.iter().enumerate() {
        if let Err(e) = apply(&mut work, op, path) {
            eprintln!("error: edit {} of {}: {e}; nothing was written", k + 1, ops.len());
            return ExitCode::from(2);
        }
    }
    // edit.out.1: each changed file in canonical layout, when it parses.
    let mut changed = Vec::new();
    for (f, src) in work.files.iter_mut() {
        if original.get(f) == Some(src) {
            continue;
        }
        if !src.ends_with('\n') {
            src.push('\n');
        }
        if let Ok(formatted) = crate::fmt::format(src) {
            *src = formatted;
        }
        if original.get(f) != Some(src) {
            changed.push(f.clone());
        }
    }
    let saved = match save(&work) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    // edit.out.2: the new state's diagnostics.
    let Some((prog, diags)) = check_quietly(path) else {
        restore(saved);
        return ExitCode::from(2);
    };
    let errors = diags.iter().filter(|d| !d.warning).count();
    let warnings = diags.len() - errors;
    let kept = !dry && !(if_clean && errors > 0);
    if !kept {
        restore(saved);
    }
    let mut out = String::new();
    if show_diff || dry {
        for f in &changed {
            let name = f.display().to_string();
            out.push_str(&crate::fmt::diff(&name, &original[f], &work.files[f]).replacen(
                " (formatted)\n",
                " (edited)\n",
                1,
            ));
        }
    }
    print!("{out}");
    if !diags.is_empty() {
        crate::modules::report(&prog.files, diags, json);
    }
    let verb = if kept {
        "edited"
    } else if dry {
        "would edit"
    } else {
        "not written (--if-clean)"
    };
    let names: Vec<String> = changed.iter().map(|f| f.display().to_string()).collect();
    if json {
        let files: Vec<String> = names.iter().map(|n| json_str(n)).collect();
        println!(
            "{{\"summary\":true,\"edits\":{},\"written\":{kept},\"files\":[{}],\"errors\":{errors},\"warnings\":{warnings}}}",
            ops.len(),
            files.join(",")
        );
    } else {
        // edit.out.2: warnings are counted only when there are some.
        let plural = |n: usize, w: &str| format!("{n} {w}{}", if n == 1 { "" } else { "s" });
        let warned = if warnings > 0 { format!(", {}", plural(warnings, "warning")) } else { String::new() };
        println!("{verb} {}; {}{warned}", plural(changed.len(), "file"), plural(errors, "error"));
    }
    if errors > 0 {
        return ExitCode::from(2);
    }
    // edit.out.4: `--test` runs the tests of the edited file.
    if test && kept {
        let Ok(exe) = std::env::current_exe() else { return ExitCode::from(2) };
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("test").arg(path);
        if json {
            cmd.arg("--json");
        }
        return match cmd.status() {
            Ok(s) => ExitCode::from(s.code().unwrap_or(1) as u8),
            Err(e) => {
                eprintln!("error: can't run the tests: {e}");
                ExitCode::from(2)
            }
        };
    }
    ExitCode::SUCCESS
}

/// Load and check `path` without reporting; a file without `main` is fine.
fn check_quietly(path: &str) -> Option<(crate::ast::Program, Vec<crate::diag::Diagnostic>)> {
    let (prog, mut diags) = match crate::modules::load(path) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("error: {e}");
            return None;
        }
    };
    if diags.is_empty() {
        diags = crate::check::check(&prog).0;
    }
    if !Path::new(path).is_dir() {
        diags.retain(|d| d.rule != "mod.main.1");
    }
    Some((prog, diags))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_parses() {
        let j = parse_json(r#"[{"op":"x","n":-1.5,"t":true,"s":"a\n\"bA"}]"#).unwrap();
        let J::Arr(xs) = j else { panic!() };
        assert_eq!(xs[0].str("s"), Some("a\n\"bA"));
        assert_eq!(xs[0].get("n"), Some(&J::Num(-1.5)));
        assert!(parse_json("[1,").is_err());
    }

    #[test]
    fn finds_declarations() {
        let src = "use std.fs\n\n## Doc.\n@test fn t()\n    x(\n        1,\n    )\n\n# loose\n\nfn P.m(self) int\n    1\nMAX := 3\nerror enum E\n    A\n";
        let ds = decls(src);
        let got: Vec<(&str, usize, usize, usize)> =
            ds.iter().map(|d| (d.id.as_str(), d.start, d.sig_end, d.end)).collect();
        assert_eq!(
            got,
            [("fn:t", 2, 3, 7), ("method:P.m", 10, 10, 12), ("const:MAX", 12, 12, 13), ("type:E", 13, 13, 15)]
        );
    }
}
