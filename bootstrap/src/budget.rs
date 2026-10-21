//! `--budget N [--from K]` on every command (budget.out.1-6, ADR 0394):
//! the command runs as a child process, its output is split into records,
//! and about N tokens of them are shown, most severe first, with counts for
//! the rest and the `--from` handle that continues.

use std::process::{Command, ExitCode, Stdio};

/// What `--budget` and `--from` asked for.
pub struct Budget {
    pub tokens: usize,
    pub from: usize,
}

/// The default budget of each command, in estimated tokens (budget.out.5).
/// ADR 0394 records the corpus measurements behind each number.
pub fn default_for(command: &str) -> usize {
    match command {
        "check" | "fix" | "edit" => 1500,
        "test" => 1500,
        "run" => 2000,
        "explain" | "effects" | "diff" => 2000,
        _ => 4000,
    }
}

/// Remove `--budget [N]` and `--from K` from `args` (the words before a
/// program's own arguments). `Ok(None)` when neither is present. `api --card`
/// keeps its own `--budget` (ADR 0311).
pub fn take(args: &mut Vec<String>) -> Result<Option<Budget>, String> {
    if args.first().is_some_and(|c| c == "api") && args.iter().any(|a| a == "--card") {
        return Ok(None);
    }
    let mut command = args.iter().find(|a| !a.starts_with("--")).cloned().unwrap_or_default();
    // api.diff.5: `api --diff` takes the budget of `diff`.
    if command == "api" && args.iter().any(|a| a == "--diff") {
        command = "diff".to_string();
    }
    let mut tokens = None;
    let mut from = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--budget" => {
                // budget.out.5: a bare `--budget` takes the command's default.
                let n = args.get(i + 1).filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()));
                match n.map(|v| v.parse::<usize>()) {
                    Some(Ok(n)) => {
                        tokens = Some(n);
                        args.drain(i..i + 2);
                    }
                    Some(Err(_)) => return Err("`--budget` is too large (budget.out.1)".into()),
                    None => {
                        tokens = Some(default_for(&command));
                        args.remove(i);
                    }
                }
            }
            "--from" => {
                let Some(k) = args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) else {
                    return Err(
                        "`--from` needs the record number a budgeted output printed, e.g. `--from 12` (budget.out.4)"
                            .into(),
                    );
                };
                from = Some(k);
                args.drain(i..i + 2);
            }
            _ => i += 1,
        }
    }
    if tokens.is_none() && from.is_none() {
        return Ok(None);
    }
    Ok(Some(Budget { tokens: tokens.unwrap_or_else(|| default_for(&command)), from: from.unwrap_or(0) }))
}

/// Run `ail ARGS` as a child with the budget applied to its output
/// (budget.out.1). Standard input passes through; the exit status is the
/// child's.
pub fn run(budget: &Budget, args: &[String], json: bool) -> ExitCode {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: can't find the `ail` executable to apply `--budget`: {e}");
            return ExitCode::from(2);
        }
    };
    let out = match Command::new(exe).args(args).stdin(Stdio::inherit()).output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("error: can't run `ail` under `--budget`: {e}");
            return ExitCode::from(2);
        }
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let (o, e) = apply(budget, &stdout, &stderr, json);
    print!("{o}");
    eprint!("{e}");
    ExitCode::from(out.status.code().map_or(1, |c| c.clamp(0, 255) as u8))
}

/// One record of a command's output (budget.out.2).
#[derive(Debug)]
struct Record {
    text: String,
    rank: u8,
    kind: &'static str,
    stderr: bool,
}

/// The rank order (budget.out.3): errors and traps, failed tests, warnings,
/// other lines, passed tests; summaries are always shown.
const KINDS: [&str; 7] = ["error", "fail", "breaking", "warning", "line", "ok", "summary"];

fn rank_of(kind: &'static str) -> u8 {
    KINDS.iter().position(|k| *k == kind).unwrap_or(4) as u8
}

/// Split one stream into records (budget.out.2).
fn records(text: &str, stderr: bool, out: &mut Vec<Record>) {
    let mut push = |text: String, kind: &'static str| out.push(Record { text, rank: rank_of(kind), kind, stderr });
    let mut current: Option<(String, &'static str)> = None;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches('\n');
        let continues = body.starts_with(' ')
            || body.starts_with('\t')
            || ["help: ", "note: ", "fix: "].iter().any(|p| body.starts_with(p));
        if continues && let Some((t, _)) = current.as_mut() {
            t.push_str(line);
            continue;
        }
        if let Some((t, k)) = current.take() {
            push(t, k);
        }
        let kind = classify(body);
        if matches!(kind, "error" | "fail" | "breaking" | "warning") && !body.starts_with('{') {
            current = Some((line.to_string(), kind));
        } else {
            push(line.to_string(), kind);
        }
    }
    if let Some((t, k)) = current {
        push(t, k);
    }
}

/// The kind of a record from its first line (budget.out.2).
fn classify(line: &str) -> &'static str {
    if line.starts_with('{') {
        let has = |s: &str| line.contains(s);
        return if has("\"summary\":true") {
            "summary"
        } else if has("\"severity\":\"error\"") || has("\"severity\":\"trap\"") {
            "error"
        } else if has("\"status\":\"fail\"") {
            "fail"
        } else if has("\"breaking\":[\"") {
            "breaking"
        } else if has("\"severity\":\"warning\"") {
            "warning"
        } else if has("\"status\":\"pass\"") {
            "ok"
        } else {
            "line"
        };
    }
    if line.starts_with("FAIL ") {
        return "fail";
    }
    if line.starts_with("ok ") {
        return "ok";
    }
    if is_summary(line) {
        return "summary";
    }
    // api.diff.5: a change that breaks callers comes before the other changes.
    if line.contains("  breaking: ") && line.starts_with(['+', '-', '~']) {
        return "breaking";
    }
    // `PATH:LINE:COL: SEVERITY[RULE]: message`
    let mut parts = line.splitn(4, ':');
    let (_, l, c, rest) = (parts.next(), parts.next(), parts.next(), parts.next());
    let num = |s: Option<&str>| s.is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()));
    if num(l) && num(c) {
        let rest = rest.unwrap_or("").trim_start();
        if rest.starts_with("error[") || rest.starts_with("trap[") {
            return "error";
        }
        if rest.starts_with("warning[") {
            return "warning";
        }
    }
    "line"
}

/// `N passed, M failed` from `ail test`.
fn is_summary(line: &str) -> bool {
    let w: Vec<&str> = line.split(' ').collect();
    matches!(w.as_slice(), [p, "passed,", f, "failed"] if p.parse::<u64>().is_ok() && f.parse::<u64>().is_ok())
}

/// The budgeted stdout and stderr (budget.out.1-4).
fn apply(budget: &Budget, stdout: &str, stderr: &str, json: bool) -> (String, String) {
    let total_tokens = crate::api::card::estimate_tokens(stdout) + crate::api::card::estimate_tokens(stderr);
    // budget.out.1: output that fits is passed on unchanged.
    if budget.from == 0 && total_tokens <= budget.tokens {
        return (stdout.to_string(), stderr.to_string());
    }
    let mut recs = Vec::new();
    records(stdout, false, &mut recs);
    records(stderr, true, &mut recs);
    let (summaries, mut recs): (Vec<Record>, Vec<Record>) = recs.into_iter().partition(|r| r.kind == "summary");
    // budget.out.3: a stable sort keeps each rank in output order.
    recs.sort_by_key(|r| r.rank);
    let total = recs.len();
    let mut used: usize = summaries.iter().map(|r| crate::api::card::estimate_tokens(&r.text)).sum();
    let mut shown = budget.from.min(total);
    let (mut o, mut e) = (String::new(), String::new());
    for r in &recs[shown..] {
        let cost = crate::api::card::estimate_tokens(&r.text);
        // At least one record is shown, so a large one can't stall `--from`.
        if used + cost > budget.tokens && shown > budget.from {
            break;
        }
        used += cost;
        shown += 1;
        if r.stderr { &mut e } else { &mut o }.push_str(&r.text);
    }
    // Everything fit after all (one record over the budget): unchanged.
    if budget.from == 0 && shown == total {
        return (stdout.to_string(), stderr.to_string());
    }
    for r in &summaries {
        if r.stderr { &mut e } else { &mut o }.push_str(&r.text);
    }
    // budget.out.4: what was left out, by kind, and how to continue.
    let mut counts: Vec<(&str, usize)> = Vec::new();
    for r in recs[..budget.from.min(total)].iter().chain(&recs[shown..]) {
        match counts.iter_mut().find(|(k, _)| *k == r.kind) {
            Some((_, n)) => *n += 1,
            None => counts.push((r.kind, 1)),
        }
    }
    counts.sort_by_key(|(k, _)| rank_of(k));
    let first_left = recs.get(shown).or(recs.first());
    let note = note(budget, total, shown, total_tokens, &counts, json);
    if first_left.is_some_and(|r| r.stderr) {
        e.push_str(&note);
    } else {
        o.push_str(&note);
    }
    (o, e)
}

/// The closing note (budget.out.4).
fn note(budget: &Budget, total: usize, shown: usize, tokens: usize, counts: &[(&str, usize)], json: bool) -> String {
    let plural = |k: &str, n: usize| {
        let name = match k {
            "line" => "line",
            "ok" => "passed test",
            "fail" => "failed test",
            k => k,
        };
        format!("{n} {name}{}", if n == 1 { "" } else { "s" })
    };
    let next = (shown < total).then_some(shown);
    if json {
        let omitted: Vec<String> = counts.iter().map(|(k, n)| format!("\"{k}\":{n}")).collect();
        let next = next.map_or("null".to_string(), |n| n.to_string());
        format!(
            "{{\"budget\":{},\"from\":{},\"shown\":{},\"total\":{total},\"tokens\":{tokens},\"omitted\":{{{}}},\"next\":{next}}}\n",
            budget.tokens,
            budget.from,
            shown - budget.from.min(total),
            omitted.join(",")
        )
    } else {
        let left: Vec<String> = counts.iter().map(|(k, n)| plural(k, *n)).collect();
        let left = if left.is_empty() { "nothing".to_string() } else { left.join(", ") };
        let next = next.map_or(String::new(), |n| format!("; continue with --from {n}"));
        if budget.from >= total {
            return format!("[budget {}: no records from {}; the output has {total}]\n", budget.tokens, budget.from);
        }
        format!(
            "[budget {}: records {}-{} of {total} shown (~{tokens} tokens in all); not shown: {left}{next}]\n",
            budget.tokens,
            budget.from.min(total) + 1,
            shown
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(tokens: usize, from: usize) -> Budget {
        Budget { tokens, from }
    }

    #[test]
    fn fitting_output_is_unchanged() {
        let (o, e) = apply(&b(1000, 0), "ok a\n1 passed, 0 failed\n", "", false);
        assert_eq!((o.as_str(), e.as_str()), ("ok a\n1 passed, 0 failed\n", ""));
    }

    #[test]
    fn errors_come_first_and_the_rest_is_counted() {
        let err =
            "f.ail:3:1: warning[x.y.1]: w\n    code\nf.ail:9:2: error[a.b.1]: bad\n    code\n    ^\nhelp: do it\n";
        let (o, e) = apply(&b(12, 0), "", err, false);
        assert_eq!(o, "");
        assert!(e.starts_with("f.ail:9:2: error[a.b.1]: bad\n    code\n    ^\nhelp: do it\n"), "{e}");
        assert!(e.ends_with("not shown: 1 warning; continue with --from 1]\n"), "{e}");
        let (_, e) = apply(&b(12, 1), "", err, false);
        assert!(e.starts_with("f.ail:3:1: warning[x.y.1]: w\n    code\n[budget 12: records 2-2 of 2"), "{e}");
    }

    #[test]
    fn json_note_and_summary() {
        let out = "{\"test\":\"a\",\"status\":\"pass\"}\n{\"test\":\"b\",\"status\":\"fail\",\"message\":\"m\"}\n{\"summary\":true,\"passed\":1,\"failed\":1}\n";
        let (o, _) = apply(&b(1, 0), out, "", true);
        let lines: Vec<&str> = o.lines().collect();
        assert!(lines[0].contains("\"b\""));
        assert!(lines[1].starts_with("{\"summary\":true"));
        assert!(lines[2].starts_with("{\"budget\":1,\"from\":0,\"shown\":1,\"total\":2,\"tokens\":"), "{}", lines[2]);
        assert!(lines[2].ends_with(",\"omitted\":{\"ok\":1},\"next\":1}"), "{}", lines[2]);
    }
}
