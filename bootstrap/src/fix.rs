//! `ail fix FILE [--dry-run] [--json]` (fix.apply.1-4, ADR 0203): apply the
//! machine-applicable fixes that diagnostics carry (ADR 0145), re-check, and
//! repeat until nothing fixable is left.

use crate::diag::{self, Diagnostic, Pos};
use std::process::ExitCode;

/// Rounds of fix-and-recheck before giving up (fix.apply.2).
const MAX_ROUNDS: usize = 5;

/// Lex, parse and check `src`; the diagnostics are settled (sorted, spans and
/// anchored edits resolved).
fn diagnose(src: &str) -> Vec<Diagnostic> {
    let (tokens, mut diags) = crate::lexer::lex(src);
    let (program, parse_diags) = crate::parser::parse_with_prelude(tokens);
    diags.extend(parse_diags);
    let bad_lines: Vec<usize> = diags.iter().filter(|d| d.rule.starts_with("lex.str.")).map(|d| d.pos.line).collect();
    diags.retain(|d| d.rule.starts_with("lex.str.") || !bad_lines.contains(&d.pos.line));
    if diags.is_empty() {
        diags = crate::check::check(&program).0;
    }
    diag::settle(&mut diags, src);
    diags
}

/// A fix chosen for application: its edits as character offsets.
struct Chosen {
    rule: &'static str,
    pos: Pos,
    title: String,
    edits: Vec<(usize, usize, String)>,
}

/// The first fix of every diagnostic that has one, skipping a fix whose edits
/// overlap an earlier-chosen one (fix.apply.1).
fn choose(diags: &[Diagnostic], src: &str) -> Vec<Chosen> {
    let mut starts = vec![0usize];
    let chars: Vec<char> = src.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if *c == '\n' {
            starts.push(i + 1);
        }
    }
    let offset = |p: Pos| starts.get(p.line.wrapping_sub(1)).map(|s| s + p.col - 1).filter(|o| *o <= chars.len());
    let mut chosen: Vec<Chosen> = Vec::new();
    let mut taken: Vec<(usize, usize)> = Vec::new();
    for d in diags {
        let Some(fix) = d.fixes.first() else { continue };
        let edits: Option<Vec<(usize, usize, String)>> =
            fix.edits.iter().map(|e| Some((offset(e.pos)?, offset(e.end)?, e.text.clone()))).collect();
        let Some(edits) = edits else { continue };
        let clash = edits.iter().any(|(s, e, _)| taken.iter().any(|(ts, te)| s <= te && ts <= e));
        if clash {
            continue;
        }
        taken.extend(edits.iter().map(|(s, e, _)| (*s, *e)));
        chosen.push(Chosen { rule: d.rule, pos: d.pos, title: fix.title.clone(), edits });
    }
    chosen
}

/// Apply the edits from last to first so earlier offsets stay valid.
fn apply(src: &str, chosen: &[Chosen]) -> String {
    let mut chars: Vec<char> = src.chars().collect();
    let mut all: Vec<&(usize, usize, String)> = chosen.iter().flat_map(|c| &c.edits).collect();
    all.sort_by_key(|(s, e, _)| (*s, *e));
    for (s, e, text) in all.into_iter().rev() {
        chars.splice(*s..*e, text.chars());
    }
    chars.into_iter().collect()
}

pub fn command(args: &[&str], json: bool) -> ExitCode {
    let dry = args.contains(&"--dry-run");
    if let Some(bad) = args.iter().find(|a| a.starts_with("--") && **a != "--dry-run") {
        eprintln!("error: unknown option {bad} for `ail fix`");
        return ExitCode::from(2);
    }
    let files: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with("--")).collect();
    let [path] = files.as_slice() else {
        eprintln!("usage: ail fix FILE [--dry-run] [--json]");
        return ExitCode::from(2);
    };
    let original = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: can't read {path}: {e}");
            return ExitCode::from(2);
        }
    };
    let canonical = crate::fmt::format(&original).is_ok_and(|f| f == original);
    let mut src = original.clone();
    let mut applied = 0;
    let mut rounds = 0;
    let mut out = String::new();
    for round in 1..=MAX_ROUNDS {
        let chosen = choose(&diagnose(&src), &src);
        if chosen.is_empty() {
            break;
        }
        rounds = round;
        for c in &chosen {
            applied += 1;
            if json {
                out.push_str(&format!(
                    "{{\"api\":{},\"applied\":{{\"round\":{round},\"rule\":{},\"line\":{},\"col\":{},\"title\":{}}}}}\n",
                    diag::API_VERSION,
                    diag::json_str(c.rule),
                    c.pos.line,
                    c.pos.col,
                    diag::json_str(&c.title)
                ));
            } else {
                out.push_str(&format!("fixed {path}:{}:{}: [{}] {}\n", c.pos.line, c.pos.col, c.rule, c.title));
            }
        }
        src = apply(&src, &chosen);
    }
    // fix.apply.3: a file that was canonical stays canonical.
    if applied > 0 && canonical {
        if let Ok(f) = crate::fmt::format(&src) {
            src = f;
        }
    }
    let mut left = diagnose(&src);
    let errors = left.iter().filter(|d| !d.warning).count();
    if applied > 0 && src != original {
        if dry {
            if !json {
                out.push_str(&crate::fmt::diff(path, &original, &src));
            }
        } else if let Err(e) = std::fs::write(path, &src) {
            eprintln!("error: can't write {path}: {e}");
            return ExitCode::from(2);
        }
    }
    if json {
        out.push_str(&format!(
            "{{\"api\":{},\"summary\":true,\"applied\":{applied},\"rounds\":{rounds},\"dry_run\":{dry},\"clean\":{},\"errors\":{errors}}}\n",
            diag::API_VERSION,
            errors == 0
        ));
    } else {
        let verb = if dry { "would apply" } else { "applied" };
        out.push_str(&format!("{verb} {applied} fix(es) in {rounds} round(s); {errors} error(s) remain\n"));
    }
    print!("{out}");
    if !left.is_empty() {
        diag::report(&mut left, path, &src, json);
    }
    if errors == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) }
}
