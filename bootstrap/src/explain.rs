//! `ail explain RULE [--json]`: a spec rule's text, the ADRs it cites and the
//! nearest example, from the specification embedded at build time (ADR 0146).
//! A repair then needs no search of `docs/spec/`.

use std::fmt::Write as _;
use std::process::ExitCode;

include!(concat!(env!("OUT_DIR"), "/docs.rs"));

/// The code for an implementation limit; not a rule ID (spec README, rule 6).
const UNSUPPORTED: &str = "bootstrap.unsupported";
const UNSUPPORTED_TEXT: &str = "Not a spec rule: the bootstrap doesn't implement this construct yet. The program \
may be valid AIL. The message names the construct and the spec rule that defines it; run `ail explain` on that \
rule for the full text. Rewrite the code without the construct, or wait for the bootstrap to support it. \
(spec README, Rule IDs, item 6; ADR 0023)";

pub struct Explanation {
    pub rule: String,
    /// `docs/spec/NAME.md` and the 1-based line of the rule.
    pub file: String,
    pub line: usize,
    pub text: String,
    /// ADR number and title, in order of first citation.
    pub adrs: Vec<(String, String)>,
    pub example: Option<String>,
}

/// Is `s` shaped like a rule ID, `chapter.section.n`?
fn is_rule_id(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 3
        && parts[..2].iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_lowercase() || c == '_'))
        && !parts[2].is_empty()
        && parts[2].chars().all(|c| c.is_ascii_digit())
}

/// The rule ID that starts `line`, if it starts one (`**chapter.section.n** — ...`).
fn rule_at(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("**")?;
    let id = &rest[..rest.find("**")?];
    is_rule_id(id).then_some(id)
}

pub fn all_rules() -> Vec<&'static str> {
    SPEC.iter().flat_map(|(_, text)| text.lines().filter_map(rule_at)).collect()
}

pub fn explain(rule: &str) -> Option<Explanation> {
    if rule == UNSUPPORTED {
        return Some(Explanation {
            rule: rule.into(),
            file: "docs/spec/README.md".into(),
            line: SPEC
                .iter()
                .find(|(n, _)| *n == "README.md")
                .and_then(|(_, t)| t.lines().position(|l| l.contains(UNSUPPORTED)))
                .map_or(1, |i| i + 1),
            text: UNSUPPORTED_TEXT.into(),
            adrs: adrs_in(UNSUPPORTED_TEXT),
            example: None,
        });
    }
    for (name, src) in SPEC {
        let lines: Vec<&str> = src.lines().collect();
        let Some(start) = lines.iter().position(|l| rule_at(l) == Some(rule)) else { continue };
        // The rule's text runs to the next rule, heading or code block.
        let mut end = start + 1;
        while end < lines.len() {
            let l = lines[end];
            if rule_at(l).is_some() || l.starts_with('#') || l.starts_with("```") {
                break;
            }
            end += 1;
        }
        let first = lines[start].split_once("** ").map_or(lines[start], |(_, t)| t);
        let first = first.trim_start_matches(['—', '-', ' ']);
        let text = std::iter::once(first).chain(lines[start + 1..end].iter().copied()).collect::<Vec<_>>();
        let text = text.join("\n").trim().to_string();
        // The nearest `ail` example after the rule, within the same section.
        let mut example = None;
        let mut i = end;
        while i < lines.len() && !lines[i].starts_with('#') {
            if lines[i].trim().starts_with("```ail") {
                let body: Vec<&str> = lines[i + 1..].iter().take_while(|l| !l.starts_with("```")).copied().collect();
                example = Some(body.join("\n"));
                break;
            }
            i += 1;
        }
        return Some(Explanation {
            rule: rule.into(),
            file: format!("docs/spec/{name}"),
            line: start + 1,
            adrs: adrs_in(&text),
            text,
            example,
        });
    }
    None
}

/// The ADRs cited in `text` (`ADR NNNN`), with titles from the ADR index.
fn adrs_in(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (i, _) in text.match_indices("ADR ") {
        let num: String = text[i + 4..].chars().take_while(char::is_ascii_digit).collect();
        if num.len() != 4 || out.iter().any(|(n, _)| *n == num) {
            continue;
        }
        let row = format!("| [{num}](");
        let title = ADR_INDEX
            .lines()
            .find(|l| l.starts_with(&row))
            .and_then(|l| l.split(" | ").nth(1))
            .unwrap_or("")
            .to_string();
        out.push((num, title));
    }
    out
}

/// Closest known rule IDs to a mistyped one, for the "did you mean" help.
fn near(rule: &str) -> Vec<&'static str> {
    let prefix = rule.rsplit_once('.').map_or(rule, |(p, _)| p);
    let mut same: Vec<&str> = all_rules().into_iter().filter(|r| r.starts_with(&format!("{prefix}."))).collect();
    if same.is_empty() {
        let chapter = rule.split('.').next().unwrap_or("");
        // One rule per section of the same chapter.
        let mut sections: Vec<&str> = Vec::new();
        for r in all_rules() {
            let section = r.rsplit_once('.').map_or(r, |(p, _)| p);
            if r.split('.').next() == Some(chapter) && !sections.contains(&section) {
                sections.push(section);
                same.push(r);
            }
        }
        same.sort();
    }
    same.truncate(8);
    same
}

pub fn run(rule: &str, json: bool) -> ExitCode {
    match explain(rule) {
        Some(e) if json => {
            println!("{}", to_json(&e));
            ExitCode::SUCCESS
        }
        Some(e) => {
            print!("{}", to_text(&e));
            ExitCode::SUCCESS
        }
        None => {
            let near = near(rule);
            let help = if near.is_empty() {
                "rule IDs look like `expr.bind.4`; see docs/spec/".to_string()
            } else {
                format!("known rules nearby: {}", near.join(", "))
            };
            if json {
                eprintln!(
                    "{{\"api\":1,\"severity\":\"error\",\"message\":{},\"help\":{}}}",
                    json_str(&format!("unknown rule `{rule}`")),
                    json_str(&help)
                );
            } else {
                eprintln!("error: unknown rule `{rule}`\nhelp: {help}");
            }
            ExitCode::from(2)
        }
    }
}

fn to_text(e: &Explanation) -> String {
    let mut s = format!("{} ({}:{})\n\n{}\n", e.rule, e.file, e.line, e.text);
    if !e.adrs.is_empty() {
        s.push('\n');
        for (n, t) in &e.adrs {
            let _ = writeln!(s, "ADR {n}: {t}");
        }
    }
    if let Some(x) = &e.example {
        let _ = write!(s, "\nexample:\n");
        for l in x.lines() {
            let _ = writeln!(s, "    {l}");
        }
    }
    s
}

fn to_json(e: &Explanation) -> String {
    let mut s = format!("{{\"api\":1,\"rule\":{}", json_str(&e.rule));
    let _ = write!(s, ",\"file\":{},\"line\":{}", json_str(&e.file), e.line);
    let _ = write!(s, ",\"text\":{}", json_str(&e.text));
    s.push_str(",\"adrs\":[");
    for (i, (n, t)) in e.adrs.iter().enumerate() {
        let sep = if i > 0 { "," } else { "" };
        let _ = write!(s, "{sep}{{\"id\":{},\"title\":{}}}", json_str(n), json_str(t));
    }
    s.push(']');
    if let Some(x) = &e.example {
        let _ = write!(s, ",\"example\":{}", json_str(x));
    }
    s.push('}');
    s
}

fn json_str(v: &str) -> String {
    crate::diag::json_str(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rule_id_is_unique() {
        let mut rules = all_rules();
        let n = rules.len();
        rules.sort();
        rules.dedup();
        assert_eq!(rules.len(), n, "a rule ID appears twice in docs/spec");
    }

    #[test]
    fn explains_a_rule_with_its_adr_and_example() {
        let e = explain("expr.bind.4").unwrap();
        assert!(e.text.starts_with("Declaring a local name"), "{}", e.text);
        assert!(e.adrs.iter().any(|(n, t)| n == "0039" && !t.is_empty()));
        assert!(e.example.unwrap().contains("mut sum := 0"));
    }

    #[test]
    fn rule_ids_are_recognised() {
        assert!(is_rule_id("expr.hole.1"));
        assert!(!is_rule_id("expr.hole"));
        assert!(!is_rule_id("Expr.hole.1"));
        assert!(explain("expr.nope.1").is_none());
        assert!(explain(UNSUPPORTED).is_some());
    }
}
