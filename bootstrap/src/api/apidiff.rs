//! `ail api --diff OLD NEW [--json]`: what changed in the public API between
//! two versions of a file or package (api.diff.1-5, ADR 0729). Declarations
//! are matched by id (api.id.1); only the public ones count, and a change
//! that can break a caller is marked `breaking:`.

use std::collections::{HashMap, HashSet};
use std::process::ExitCode;

use super::diff::{KEYS, at_revision};
use super::{Api, Item};
use crate::check::{self, Resolved};
use crate::diag::json_str;
use crate::docs;
use crate::modules;

const USAGE: &str = "usage: ail api --diff OLD NEW [--json]   (two files, or two package directories)\n       ail api --diff --rev REV PATH [--json]   (PATH as of git revision REV against now)\n";

/// The compared keys of a public declaration (api.diff.2); a body is not part of an API.
const API_KEYS: [&str; 6] = ["signature", "effects", "fails_with", "requires", "ensures", "default"];

/// One public declaration: its id, kind and the compared properties as text.
struct Decl {
    id: String,
    kind: &'static str,
    props: Vec<(&'static str, String)>,
}

impl Decl {
    fn get(&self, key: &str) -> &str {
        self.props.iter().find(|(k, _)| *k == key).map_or("", |(_, v)| v.as_str())
    }
}

pub fn command(args: &[&str], json: bool) -> ExitCode {
    let mut paths = Vec::new();
    let mut rev = None;
    let mut it = args.iter().copied().filter(|a| *a != "--diff");
    while let Some(a) = it.next() {
        match a {
            "--rev" => match it.next() {
                Some(r) => rev = Some(r),
                None => {
                    eprint!("error: `--rev` needs a git revision, e.g. `--rev HEAD` (api.diff.1)\n{USAGE}");
                    return ExitCode::from(2);
                }
            },
            "--json" => {}
            a if a.starts_with("--") => {
                eprint!("error: unknown option `{a}`\n{USAGE}");
                return ExitCode::from(2);
            }
            a => paths.push(a),
        }
    }
    let checkout;
    let (old, new) = match (rev, paths.as_slice()) {
        (None, [old, new]) => (old.to_string(), new.to_string()),
        (Some(r), [path]) => match at_revision(r, path) {
            Ok(dir) => {
                checkout = dir;
                (checkout.path.clone(), path.to_string())
            }
            Err(e) => {
                eprintln!("error: {e} (api.diff.1)");
                return ExitCode::from(2);
            }
        },
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let (a, b) = match (public_api(&old), public_api(&new)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let (out, changes) = render(&old, &new, &a, &b, json);
    print!("{out}");
    // As `ail diff`: 0 when the API is the same, 1 when it changed, 2 for an error.
    if changes == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}

/// Load and check `path` and list its public declarations (api.diff.2). A
/// version that does not check cannot be compared (api.diff.4).
fn public_api(path: &str) -> Result<Vec<Decl>, String> {
    let dir = std::path::Path::new(path).is_dir();
    let (mut prog, mut diags) = modules::load(path).map_err(|e| e.to_string())?;
    // A single file may lack `main`: it is a library when compared (api.diff.4).
    diags.retain(|d| !(!dir && d.rule == "mod.main.1"));
    let mut resolved = Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&prog);
        diags.retain(|d| !(!dir && d.rule == "mod.main.1"));
    }
    if diags.iter().any(|d| !d.warning) {
        return Err(format!("{path} has errors, so its API can't be compared; fix them first (api.diff.4)"));
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
    api.whole = dir;
    let mut all = Vec::new();
    for item in api.outline_items() {
        flatten(&prog, item, false, &mut all);
    }
    let public: HashSet<String> = all.iter().filter(|(p, _)| *p).map(|(_, d)| d.id.clone()).collect();
    Ok(all
        .into_iter()
        .filter(|(p, d)| {
            // An `impl` is part of the API when its type is (api.diff.2).
            if d.kind == "impl" {
                let ty = d.id.strip_prefix("impl:").and_then(|r| r.split('.').next()).unwrap_or("");
                return public.contains(&format!("type:{ty}"));
            }
            *p
        })
        .map(|(_, d)| d)
        .collect())
}

/// Adds `item` and its members, each with whether it is public: written
/// `pub`, or the variant of a public enum, a field of such a variant, or a
/// method of a public trait.
fn flatten(prog: &crate::ast::Program, item: Item, inherited: bool, out: &mut Vec<(bool, Decl)>) {
    let public = prog.is_pub(item.pos) || inherited;
    let mut props = vec![("signature", item.signature)];
    for (k, v) in item.extra {
        if API_KEYS.contains(&k) && KEYS.contains(&k) {
            let mut s = String::new();
            v.write(&mut s, None);
            props.push((k, s));
        }
    }
    let kind = item.kind;
    out.push((public && kind != "impl", Decl { id: item.id, kind, props }));
    // An impl is decided afterwards, by its type; its methods repeat the trait's.
    let skip_members = kind == "impl";
    for (group, members) in item.members {
        let inherit =
            public && matches!((kind, group), ("enum", "variants") | ("variant", "fields") | ("trait", "methods"));
        for m in members {
            if skip_members {
                continue;
            }
            flatten(prog, m, inherit, out);
        }
    }
}

/// The quoted strings of a JSON array written as text: `["a","b"]`.
fn set_of(v: &str) -> HashSet<&str> {
    v.split('"').skip(1).step_by(2).collect()
}

/// Why `new` may break a caller of `old`, for one compared key (api.diff.3).
fn breaks(key: &str, old: &str, new: &str) -> bool {
    match key {
        "signature" => true,
        // A new effect or error type is something a caller must now allow for.
        "effects" | "fails_with" => !set_of(new).is_subset(&set_of(old)),
        // A new or changed precondition is stricter; dropping one is not.
        "requires" => !set_of(new).is_empty() && set_of(new) != set_of(old),
        // A guarantee that went away was relied on.
        "ensures" => !set_of(old).is_subset(&set_of(new)),
        // A trait method that lost its default breaks implementers that relied on it.
        "default" => old == "true" && new != "true",
        _ => false,
    }
}

/// Why adding `d` can break existing code (api.diff.3), if it can.
fn added_breaks(d: &Decl) -> Option<&'static str> {
    match d.kind {
        "variant" => Some("new variant"),
        "method" if d.get("default") == "false" => Some("new required trait method"),
        _ => None,
    }
}

fn render(old: &str, new: &str, a: &[Decl], b: &[Decl], json: bool) -> (String, usize) {
    let old_by: HashMap<&str, &Decl> = a.iter().map(|d| (d.id.as_str(), d)).collect();
    let new_ids: HashSet<&str> = b.iter().map(|d| d.id.as_str()).collect();
    let (mut added, mut removed, mut changed, mut breaking) = (0, 0, 0, 0);
    let mut lines: Vec<String> = Vec::new();
    for d in b {
        match old_by.get(d.id.as_str()) {
            None => {
                added += 1;
                let why = added_breaks(d);
                breaking += usize::from(why.is_some());
                lines.push(decl_line("added", d, why.map(|w| vec![w]), json));
            }
            Some(o) => {
                let diffs: Vec<(&str, &str, &str)> =
                    API_KEYS.iter().filter(|k| o.get(k) != d.get(k)).map(|k| (*k, o.get(k), d.get(k))).collect();
                if !diffs.is_empty() {
                    changed += 1;
                    let why: Vec<&str> = diffs.iter().filter(|(k, x, y)| breaks(k, x, y)).map(|(k, _, _)| *k).collect();
                    breaking += usize::from(!why.is_empty());
                    lines.push(changed_line(d, &diffs, &why, json));
                }
            }
        }
    }
    for d in a.iter().filter(|d| !new_ids.contains(d.id.as_str())) {
        removed += 1;
        breaking += 1;
        lines.push(decl_line("removed", d, Some(vec!["removed"]), json));
    }
    let mut out = String::new();
    for l in &lines {
        out.push_str(l);
        out.push('\n');
    }
    if json {
        out.push_str(&format!(
            "{{\"summary\":true,\"old\":{},\"new\":{},\"added\":{added},\"removed\":{removed},\"changed\":{changed},\"breaking\":{breaking}}}\n",
            json_str(old),
            json_str(new)
        ));
    } else {
        out.push_str(&format!("{added} added, {removed} removed, {changed} changed; {breaking} breaking\n"));
    }
    (out, added + removed + changed)
}

fn decl_line(op: &str, d: &Decl, why: Option<Vec<&str>>, json: bool) -> String {
    let sig = d.get("signature");
    if json {
        let marker = why.map_or(String::new(), |w| {
            format!(",\"breaking\":[{}]", w.iter().map(|x| json_str(x)).collect::<Vec<_>>().join(","))
        });
        return format!(
            "{{\"change\":\"api\",\"op\":\"{op}\",\"id\":{},\"kind\":\"{}\",\"signature\":{}{marker}}}",
            json_str(&d.id),
            d.kind,
            json_str(sig)
        );
    }
    let marker = why.map_or(String::new(), |w| format!("  breaking: {}", w.join(", ")));
    format!("{} {} {sig}{marker}", if op == "added" { '+' } else { '-' }, d.id)
}

fn changed_line(d: &Decl, diffs: &[(&str, &str, &str)], why: &[&str], json: bool) -> String {
    if json {
        let keys: Vec<String> = diffs
            .iter()
            .map(|(k, o, n)| format!("\"{k}\":{{\"old\":{},\"new\":{}}}", json_str(o), json_str(n)))
            .collect();
        let marker = if why.is_empty() {
            String::new()
        } else {
            format!(",\"breaking\":[{}]", why.iter().map(|x| json_str(x)).collect::<Vec<_>>().join(","))
        };
        return format!(
            "{{\"change\":\"api\",\"op\":\"changed\",\"id\":{},\"kind\":\"{}\",\"keys\":{{{}}}{marker}}}",
            json_str(&d.id),
            d.kind,
            keys.join(",")
        );
    }
    let mut s = format!("~ {}", d.id);
    if !why.is_empty() {
        s.push_str(&format!("  breaking: {}", why.join(", ")));
    }
    for (k, o, n) in diffs {
        s.push_str(&format!("\n    {k}: {} -> {}", shown(o), shown(n)));
    }
    s
}

fn shown(v: &str) -> &str {
    if v.is_empty() { "(none)" } else { v }
}
