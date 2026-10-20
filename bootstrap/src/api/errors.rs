//! `ail check --errors FILE` (err.paths.1, ADR 0286): for every function, the
//! error types it can fail with and each place where it fails, passes a
//! failure on or handles one, as JSON. The checker records the paths
//! (`check::errpaths`); this file selects and prints them.

use std::process::ExitCode;

use super::*;

/// The report for `src`, or the diagnostics that stopped it.
pub fn report(path: &str, src: &str) -> Result<String, Vec<diag::Diagnostic>> {
    let (prog, mut diags) = crate::modules::load_src(path, src);
    let mut resolved = Resolved::default();
    if diags.is_empty() {
        (diags, resolved) = check::check(&prog);
    }
    diags.retain(|d| d.rule != "mod.main.1");
    if diags.iter().any(|d| !d.warning) {
        return Err(diags);
    }
    let mut api = Api::new(&prog, std::collections::BTreeMap::new());
    api.errors = std::mem::take(&mut resolved.fn_errors);
    let strs = |xs: &[String]| Json::Arr(xs.iter().cloned().map(Json::Str).collect());
    let mut functions = Vec::new();
    for (i, f) in prog.fns.iter().enumerate() {
        if api.outside(f.pos) || !f.top_level() {
            continue;
        }
        let fails_with = api.errors.get(&i);
        let mut paths: Vec<_> = resolved.fail_paths.iter().filter(|p| p.func == i).collect();
        if fails_with.is_none() && paths.is_empty() {
            continue;
        }
        paths.sort_by_key(|p| (p.pos.line, p.pos.col));
        let paths = paths
            .into_iter()
            .map(|p| {
                let mut kvs = vec![
                    ("line", Json::Int(p.pos.line as i64)),
                    ("col", Json::Int(p.pos.col as i64)),
                    ("kind", Json::Str(p.kind.into())),
                ];
                if let Some(how) = p.how {
                    kvs.push(("how", Json::Str(how.into())));
                }
                if let Some(call) = &p.call {
                    kvs.push(("call", Json::Str(call.clone())));
                }
                kvs.push(("errors", strs(&p.errors)));
                if p.context {
                    kvs.push(("context", Json::Bool(true)));
                }
                Json::Obj(kvs)
            })
            .collect();
        let mut kvs = vec![
            ("id", Json::Str(api.fn_ids[i].clone())),
            ("name", Json::Str(f.name.clone())),
            ("line", Json::Int(f.pos.line as i64)),
        ];
        if let Some(e) = fails_with {
            kvs.push(("fails_with", strs(e)));
        }
        kvs.push(("paths", Json::Arr(paths)));
        functions.push(Json::Obj(kvs));
    }
    functions.sort_by_key(|f| match f {
        Json::Obj(kv) => match kv.get(2) {
            Some((_, Json::Int(l))) => *l,
            _ => 0,
        },
        _ => 0,
    });
    let top = Json::Obj(vec![
        ("api", Json::Int(API_VERSION)),
        ("file", Json::Str(path.to_string())),
        ("functions", Json::Arr(functions)),
    ]);
    let mut out = String::new();
    top.write(&mut out, None);
    out.push('\n');
    Ok(out)
}

/// `ail check --errors FILE`.
pub fn command(path: &str, json: bool) -> ExitCode {
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    match report(path, &src) {
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
