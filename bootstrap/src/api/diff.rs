//! `ail diff OLD NEW [--json] [--no-tests]`: what changed between two
//! versions of a file or package, by meaning (diff.out.1-6, ADR 0396):
//! declarations added, removed or changed (signature, visibility, effects,
//! errors, contracts, docs), diagnostics that appeared or went away, and
//! tests whose outcome changed. An agent reviews a change by reading this
//! instead of the whole textual diff.

use std::collections::HashMap;
use std::process::{Command, ExitCode, Stdio};

use super::{Api, Item};
use crate::check::{self, Resolved};
use crate::diag::json_str;
use crate::docs;
use crate::modules;

pub const USAGE: &str = "usage: ail diff OLD NEW [--json] [--no-tests]   (two files, or two package directories)\n       ail diff --rev REV PATH [--json] [--no-tests]   (PATH as of git revision REV against now)\n";

/// Item keys compared by `diff.out.2`; the rest repeat the signature.
const KEYS: [&str; 10] =
    ["signature", "pub", "effects", "fails_with", "requires", "ensures", "inherits", "default", "doc", "body"];

/// One declaration: its id, kind and the compared properties as text.
struct Decl {
    id: String,
    line: usize,
    kind: &'static str,
    props: Vec<(&'static str, String)>,
}

/// A diagnostic, compared by rule, file and message (lines move).
#[derive(Clone)]
struct Found {
    severity: &'static str,
    rule: &'static str,
    file: String,
    line: usize,
    message: String,
}

/// One side of the comparison.
struct Side {
    decls: Option<Vec<Decl>>,
    diags: Vec<Found>,
    /// Test name to (passed, message), in order.
    tests: Option<Vec<(String, bool, String)>>,
}

pub fn command(args: &[&str], json: bool) -> ExitCode {
    let mut paths = Vec::new();
    let mut run_tests = true;
    let mut rev = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match *a {
            "--no-tests" => run_tests = false,
            "--rev" => match it.next() {
                Some(r) => rev = Some(*r),
                None => {
                    eprint!("error: `--rev` needs a git revision, e.g. `--rev HEAD` (diff.out.5)\n{USAGE}");
                    return ExitCode::from(2);
                }
            },
            a if a.starts_with("--") => {
                eprint!("error: unknown option `{a}`\n{USAGE}");
                return ExitCode::from(2);
            }
            a => paths.push(a),
        }
    }
    // diff.out.5: `--rev REV PATH` compares PATH as git has it at REV.
    let checkout;
    let (old, new) = match (rev, paths.as_slice()) {
        (None, [old, new]) => (old.to_string(), new.to_string()),
        (Some(r), [path]) => match at_revision(r, path) {
            Ok(dir) => {
                checkout = dir;
                (checkout.path.clone(), path.to_string())
            }
            Err(e) => {
                eprintln!("error: {e} (diff.out.5)");
                return ExitCode::from(2);
            }
        },
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let (old, new) = (old.as_str(), new.as_str());
    let (a, b) = match (side(old, run_tests), side(new, run_tests)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let (out, changes) = render(old, new, &a, &b, json);
    print!("{out}");
    // diff.out.6: as diff(1), 0 when nothing changed and 1 when something did.
    if changes == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

/// Load, check and outline `path`, and run its tests (diff.out.1).
fn side(path: &str, run_tests: bool) -> Result<Side, String> {
    let root = if std::path::Path::new(path).is_dir() { format!("{}/", path.trim_end_matches('/')) } else { path.to_string() };
    let dir = std::path::Path::new(path).is_dir();
    let (mut prog, mut diags) = modules::load(path).map_err(|e| e.to_string())?;
    let mut resolved = Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&prog);
    }
    let main_optional = !dir;
    diags.retain(|d| !(main_optional && d.rule == "mod.main.1"));
    let rejected = diags.iter().any(|d| !d.warning);
    let found = diags
        .iter()
        .map(|d| {
            let (file, p) = modules::locate(&prog.files, d.pos);
            Found {
                severity: if d.warning { "warning" } else { "error" },
                rule: d.rule,
                // Relative to the side's root, so the two sides compare (diff.out.3).
                file: if file == root { String::new() } else { file.strip_prefix(&root).unwrap_or(file).to_string() },
                line: p.line,
                message: d.message.clone(),
            }
        })
        .collect();
    let decls = (!rejected).then(|| {
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
        api.whole = dir;
        let mut decls = Vec::new();
        let items = api.outline_items();
        // diff.out.2: a declaration's text runs to the next one's line; its hash says whether the body changed.
        let mut starts = Vec::new();
        fn lines_of(items: &[Item], out: &mut Vec<usize>) {
            for i in items {
                out.push(i.pos.line);
                for (_, ms) in &i.members {
                    lines_of(ms, out);
                }
            }
        }
        lines_of(&items, &mut starts);
        starts.sort_unstable();
        for item in items {
            flatten(&prog, item, &mut decls);
        }
        for d in decls.iter_mut().filter(|d| item_is_fn(d)) {
            let next = starts.iter().copied().find(|l| *l > d.line);
            let body = body_hash(&prog, d.line, next);
            d.props.push(("body", body));
        }
        decls
    });
    let tests = (run_tests && !dir && !rejected).then(|| tests_of(path)).flatten();
    Ok(Side { decls, diags: found, tests })
}

fn item_is_fn(d: &Decl) -> bool {
    matches!(d.kind, "fn" | "method")
}

/// A short hash of a declaration's source lines after `line`, up to `next`
/// (or the end of its file), with trailing blank lines dropped.
fn body_hash(prog: &crate::ast::Program, line: usize, next: Option<usize>) -> String {
    let Some(file) = prog.file_at(crate::diag::Pos { line, col: 1 }) else { return String::new() };
    let first = line - file.base;
    let lines: Vec<&str> = file.src.lines().collect();
    let last = next.filter(|n| *n > file.base).map_or(lines.len(), |n| (n - file.base - 1).min(lines.len()));
    // From the line after the declaration's own: a signature change is reported as such.
    let mut text: Vec<&str> = lines.get(first.min(last)..last).unwrap_or(&[]).to_vec();
    while text.last().is_some_and(|l| l.trim().is_empty() || l.trim_start().starts_with('#')) {
        text.pop();
    }
    // FNV-1a: stable across runs and platforms.
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.join("\n").bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
    }
    format!("{:08x}", h >> 32)
}

fn flatten(prog: &crate::ast::Program, item: Item, out: &mut Vec<Decl>) {
    let mut props = vec![("signature", item.signature), ("pub", prog.is_pub(item.pos).to_string())];
    for (k, v) in item.extra {
        if KEYS.contains(&k) {
            let mut s = String::new();
            v.write(&mut s, None);
            props.push((k, s));
        }
    }
    if let Some(d) = item.doc {
        props.push(("doc", d));
    }
    out.push(Decl { id: item.id, line: item.pos.line, kind: item.kind, props });
    for (_, members) in item.members {
        for m in members {
            flatten(prog, m, out);
        }
    }
}

/// Each test's outcome, from `ail test --json` in a child (diff.out.4).
fn tests_of(path: &str) -> Option<Vec<(String, bool, String)>> {
    let exe = std::env::current_exe().ok()?;
    let out = Command::new(exe)
        .args(["test", "--json", "--timeout", "10000", path])
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut tests = Vec::new();
    for line in text.lines() {
        let Some(name) = field(line, "test") else { continue };
        let passed = field(line, "status").as_deref() == Some("pass");
        tests.push((name, passed, field(line, "message").unwrap_or_default()));
    }
    Some(tests)
}

/// The string value of `key` in one flat JSON line that `ail test --json` wrote.
fn field(line: &str, key: &str) -> Option<String> {
    let at = line.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let mut out = String::new();
    let mut chars = line[at..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?).unwrap_or('?'));
                }
                c => out.push(c),
            },
            c => out.push(c),
        }
    }
    None
}

/// The report and the number of changes (diff.out.2-5).
fn render(old: &str, new: &str, a: &Side, b: &Side, json: bool) -> (String, usize) {
    let mut out = String::new();
    let mut lines: Vec<String> = Vec::new();
    let (mut added, mut removed, mut changed) = (0, 0, 0);
    // diff.out.2: declarations by id, in the new version's order, then the removed ones.
    match (&a.decls, &b.decls) {
        (Some(da), Some(db)) => {
            let old_by: HashMap<&str, &Decl> = da.iter().map(|d| (d.id.as_str(), d)).collect();
            let new_ids: std::collections::HashSet<&str> = db.iter().map(|d| d.id.as_str()).collect();
            for d in db {
                match old_by.get(d.id.as_str()) {
                    None => {
                        added += 1;
                        lines.push(decl_line("added", d, json));
                    }
                    Some(o) => {
                        let diffs = prop_changes(o, d);
                        if !diffs.is_empty() {
                            changed += 1;
                            lines.push(changed_line(d, &diffs, json));
                        }
                    }
                }
            }
            for d in da.iter().filter(|d| !new_ids.contains(d.id.as_str())) {
                removed += 1;
                lines.push(decl_line("removed", d, json));
            }
        }
        _ => {
            let which = if a.decls.is_none() { old } else { new };
            lines.push(if json {
                format!("{{\"change\":\"api\",\"op\":\"unavailable\",\"reason\":{}}}", json_str(&format!("{which} has errors")))
            } else {
                format!("api: not compared, {which} has errors")
            });
        }
    }
    // diff.out.3: diagnostics by rule, file and message, counted.
    let key = |f: &Found| (f.rule, f.file.clone(), f.message.clone());
    let mut left: Vec<(&'static str, String, String)> = a.diags.iter().map(|f| key(f)).collect();
    let mut gained = Vec::new();
    for f in &b.diags {
        let k = key(f);
        match left.iter().position(|x| *x == k) {
            Some(i) => {
                left.remove(i);
            }
            None => gained.push(f.clone()),
        }
    }
    let lost: Vec<&Found> = {
        let mut rest = left.clone();
        a.diags
            .iter()
            .filter(|f| match rest.iter().position(|x| *x == key(f)) {
                Some(i) => {
                    rest.remove(i);
                    true
                }
                None => false,
            })
            .collect()
    };
    let named = |f: &Found, side: &str| {
        if f.file.is_empty() {
            side.to_string()
        } else if std::path::Path::new(side).is_dir() {
            format!("{}/{}", side.trim_end_matches('/'), f.file)
        } else {
            f.file.clone()
        }
    };
    for (op, f) in gained.iter().map(|f| ("added", f)).chain(lost.iter().map(|f| ("removed", *f))) {
        let file = named(f, if op == "added" { new } else { old });
        lines.push(if json {
            format!(
                "{{\"change\":\"diagnostic\",\"op\":\"{op}\",\"severity\":\"{}\",\"rule\":{},\"file\":{},\"line\":{},\"message\":{}}}",
                f.severity,
                json_str(f.rule),
                json_str(&file),
                f.line,
                json_str(&f.message)
            )
        } else {
            let sign = if op == "added" { '+' } else { '-' };
            format!("{sign} {}[{}] {file}:{}: {}", f.severity, f.rule, f.line, f.message)
        });
    }
    // diff.out.4: tests whose outcome changed, were added or were removed.
    let mut test_changes = 0;
    if let (Some(ta), Some(tb)) = (&a.tests, &b.tests) {
        let before: HashMap<&str, bool> = ta.iter().map(|(n, p, _)| (n.as_str(), *p)).collect();
        let status = |p: bool| if p { "pass" } else { "fail" };
        let mut push = |name: &str, was: Option<bool>, now: Option<bool>, message: &str| {
            test_changes += 1;
            let show = |s: Option<bool>| s.map_or("absent", status);
            lines.push(if json {
                // A test that fails now carries `"status":"fail"`, so `--budget` ranks it first.
                let fail = if now == Some(false) { ",\"status\":\"fail\"" } else { "" };
                format!(
                    "{{\"change\":\"test\",\"test\":{},\"old\":\"{}\",\"new\":\"{}\"{fail},\"message\":{}}}",
                    json_str(name),
                    show(was),
                    show(now),
                    json_str(message)
                )
            } else {
                let tail = if message.is_empty() { String::new() } else { format!(": {message}") };
                format!("test {name}: {} -> {}{tail}", show(was), show(now))
            });
        };
        for (name, passed, message) in tb {
            match before.get(name.as_str()) {
                Some(was) if was == passed => {}
                was => push(name, was.copied(), Some(*passed), message),
            }
        }
        for (name, passed, _) in ta {
            if !tb.iter().any(|(n, _, _)| n == name) {
                push(name, Some(*passed), None, "");
            }
        }
    }
    let total = added + removed + changed + gained.len() + lost.len() + test_changes;
    let fails = |t: &Option<Vec<(String, bool, String)>>| t.as_ref().map(|t| t.iter().filter(|x| !x.1).count());
    if json {
        for l in &lines {
            out.push_str(l);
            out.push('\n');
        }
        let tests = match (fails(&a.tests), fails(&b.tests)) {
            (Some(x), Some(y)) => format!(",\"failing_before\":{x},\"failing_after\":{y}"),
            _ => String::new(),
        };
        out.push_str(&format!(
            "{{\"summary\":true,\"old\":{},\"new\":{},\"added\":{added},\"removed\":{removed},\"changed\":{changed},\"diagnostics_added\":{},\"diagnostics_removed\":{},\"tests_changed\":{test_changes}{tests}}}\n",
            json_str(old),
            json_str(new),
            gained.len(),
            lost.len()
        ));
    } else {
        for l in &lines {
            out.push_str(l);
            out.push('\n');
        }
        let tests = match (fails(&a.tests), fails(&b.tests)) {
            (Some(x), Some(y)) => format!("; failing tests {x} -> {y}"),
            _ => "; tests not run".to_string(),
        };
        out.push_str(&format!(
            "{added} added, {removed} removed, {changed} changed; diagnostics +{} -{}{tests}\n",
            gained.len(),
            lost.len()
        ));
    }
    (out, total)
}

fn prop_changes<'d>(a: &'d Decl, b: &'d Decl) -> Vec<(&'static str, &'d str, &'d str)> {
    let get = |d: &'d Decl, k: &str| d.props.iter().find(|(x, _)| *x == k).map_or("", |(_, v)| v.as_str());
    KEYS.iter().filter_map(|k| (get(a, k) != get(b, k)).then(|| (*k, get(a, k), get(b, k)))).collect()
}

fn decl_line(op: &str, d: &Decl, json: bool) -> String {
    let sig = d.props.iter().find(|(k, _)| *k == "signature").map_or("", |(_, v)| v.as_str());
    if json {
        format!(
            "{{\"change\":\"api\",\"op\":\"{op}\",\"id\":{},\"kind\":\"{}\",\"signature\":{}}}",
            json_str(&d.id),
            d.kind,
            json_str(sig)
        )
    } else {
        format!("{} {} {sig}", if op == "added" { '+' } else { '-' }, d.id)
    }
}

fn changed_line(d: &Decl, diffs: &[(&str, &str, &str)], json: bool) -> String {
    if json {
        let keys: Vec<String> = diffs
            .iter()
            .map(|(k, o, n)| format!("\"{k}\":{{\"old\":{},\"new\":{}}}", json_str(o), json_str(n)))
            .collect();
        return format!(
            "{{\"change\":\"api\",\"op\":\"changed\",\"id\":{},\"kind\":\"{}\",\"keys\":{{{}}}}}",
            json_str(&d.id),
            d.kind,
            keys.join(",")
        );
    }
    // A body or doc comment is long; say that it changed, not how.
    let flags: Vec<&str> = diffs.iter().filter(|(k, _, _)| matches!(*k, "doc" | "body")).map(|(k, _, _)| *k).collect();
    let mut s = format!("~ {}", d.id);
    if !flags.is_empty() {
        s.push_str(&format!(" ({} changed)", flags.join(", ")));
    }
    for (k, o, n) in diffs.iter().filter(|(k, _, _)| !matches!(*k, "doc" | "body")) {
        s.push_str(&format!("\n    {k}: {} -> {}", shown(o), shown(n)));
    }
    s
}

fn shown(v: &str) -> &str {
    if v.is_empty() { "(none)" } else { v }
}

/// A temporary copy of a path as git has it at a revision, removed on drop.
struct Checkout {
    dir: std::path::PathBuf,
    path: String,
}

impl Drop for Checkout {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Extract `path` (a package directory, or a file with the directory that
/// holds its modules) at `rev` into a temporary directory (diff.out.5).
fn at_revision(rev: &str, path: &str) -> Result<Checkout, String> {
    let p = std::path::Path::new(path);
    let (dir, name) = if p.is_dir() {
        (p, None)
    } else {
        (p.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(std::path::Path::new(".")), p.file_name())
    };
    let git = |args: &[&str], cwd: &std::path::Path| -> Result<Vec<u8>, String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|e| format!("can't run git: {e}"))?;
        if !out.status.success() {
            return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim()));
        }
        Ok(out.stdout)
    };
    let tmp = std::env::temp_dir().join(format!("ail-diff-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| format!("can't create {}: {e}", tmp.display()))?;
    let mut checkout = Checkout { dir: tmp.clone(), path: String::new() };
    let tar = git(&["archive", "--format=tar", rev, "."], dir)?;
    let mut untar = Command::new("tar")
        .args(["-x", "-f", "-", "-C"])
        .arg(&tmp)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("can't run tar: {e}"))?;
    if let Some(mut input) = untar.stdin.take() {
        use std::io::Write as _;
        input.write_all(&tar).map_err(|e| format!("can't unpack {rev}: {e}"))?;
    }
    untar.wait().map_err(|e| format!("can't unpack {rev}: {e}"))?;
    let target = match name {
        Some(n) => tmp.join(n),
        None => tmp.clone(),
    };
    if !target.exists() {
        return Err(format!("{path} does not exist at {rev}"));
    }
    checkout.path = target.display().to_string();
    Ok(checkout)
}
