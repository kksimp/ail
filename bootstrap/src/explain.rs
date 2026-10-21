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

/// The rules whose diagnostics carry a machine-applicable fix, one per line.
/// `tests/explain_coverage.rs` keeps it equal to what the corpora produce.
const FIXABLE: &str = include_str!("fixable.txt");

/// Does `ail fix` repair diagnostics of `rule` (in some context)?
pub fn fixable(rule: &str) -> bool {
    FIXABLE.lines().any(|l| l == rule)
}

/// `ail explain [--json]` (explain.list.1): every rule, the chapter key and
/// file it is in, and whether a fix exists; one line each, or one JSON object.
pub fn list(json: bool) -> ExitCode {
    let mut rules: Vec<(&str, &str, &str)> = Vec::new();
    for (name, text) in SPEC {
        for r in text.lines().filter_map(rule_at) {
            rules.push((r, r.split('.').next().unwrap_or(""), name));
        }
    }
    let fixes = rules.iter().filter(|(r, _, _)| fixable(r)).count();
    if json {
        let mut s = format!("{{\"api\":1,\"count\":{},\"fixable\":{fixes},\"rules\":[", rules.len());
        for (i, (r, c, f)) in rules.iter().enumerate() {
            let sep = if i > 0 { "," } else { "" };
            let _ = write!(s, "{sep}{{\"rule\":{},\"chapter\":{},\"file\":{},\"fix\":{}}}", json_str(r), json_str(c), json_str(f), fixable(r));
        }
        println!("{s}]}}");
    } else {
        for (r, _, f) in &rules {
            println!("{r} {f}{}", if fixable(r) { " fix" } else { "" });
        }
        println!("{} rules, {fixes} with a fix", rules.len());
    }
    ExitCode::SUCCESS
}

/// Longest rule text printed in the text form, in characters; a longer rule
/// stops at the last sentence that fits and points at the file for the rest.
const TEXT_CHARS: usize = 420;

/// `text` cut after the last sentence within `TEXT_CHARS`.
fn shorten(text: &str) -> String {
    if text.chars().count() <= TEXT_CHARS {
        return text.to_string();
    }
    let head: String = text.chars().take(TEXT_CHARS).collect();
    let cut = head.rfind(". ").or_else(|| head.rfind(".\n")).map_or(head.len(), |i| i + 1);
    format!("{} ... (cut; see the line above)", &head[..cut])
}

/// Longest example printed in the text form (explain.cover.4); `--json` has it whole.
const EXAMPLE_LINES: usize = 6;

pub struct Explanation {
    pub rule: String,
    /// `docs/spec/NAME.md` and the 1-based line of the rule.
    pub file: String,
    pub line: usize,
    pub text: String,
    /// ADR number and title, in order of first citation.
    pub adrs: Vec<(String, String)>,
    pub example: Option<String>,
    /// A program the rule rejects (a block tagged `error=RULE`), and the
    /// same program repaired (the plain block right after it).
    pub rejected: Option<String>,
    pub fixed: Option<String>,
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
            rejected: None,
            fixed: None,
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
        let text = strip_links(text.join("\n").trim());
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
        let (rejected, fixed) = worked_example(rule).unzip();
        // A worked example replaces the nearest example, which may belong to another rule.
        let example = if rejected.is_some() { None } else { example.filter(|x| !x.trim().is_empty()).or_else(|| tagged_example(rule)) };
        return Some(Explanation {
            rule: rule.into(),
            file: format!("docs/spec/{name}"),
            line: start + 1,
            adrs: adrs_in(&text),
            text,
            example,
            rejected,
            fixed: fixed.flatten(),
        });
    }
    None
}

/// The first block tagged `error=RULE` anywhere in the specification, and the
/// plain block that directly follows it, if any (explain.show.1).
fn worked_example(rule: &str) -> Option<(String, Option<String>)> {
    let tag = format!("error={rule}");
    for (_, src) in SPEC {
        let lines: Vec<&str> = src.lines().collect();
        let block = |at: usize| -> String {
            lines[at + 1..].iter().take_while(|l| !l.starts_with("```")).copied().collect::<Vec<_>>().join("\n")
        };
        let Some(open) = lines.iter().position(|l| l.strip_prefix("```ail ").is_some_and(|t| t.split_whitespace().any(|w| w == tag)))
        else {
            continue;
        };
        let rejected = block(open);
        let mut i = open + 1;
        while i < lines.len() && !lines[i].starts_with("```") {
            i += 1;
        }
        // Past the closing fence, to the next block unless a heading or rule comes first.
        let fixed = lines[i + 1..]
            .iter()
            .position(|l| l.starts_with('#') || rule_at(l).is_some() || l.starts_with("```"))
            .map(|k| i + 1 + k)
            .filter(|&k| matches!(lines[k].trim_end(), "```ail" | "```ail fragment"))
            .map(block);
        return Some((rejected, fixed));
    }
    None
}

/// The first block of the specification whose `rules=` tag names `rule`, when
/// the rule's own section has no example (ADR 0670). The block is checked
/// by the spec harness like any other. A `tool=` block shows its command first,
/// so the explanation says how to run the example.
fn tagged_example(rule: &str) -> Option<String> {
    for (_, src) in SPEC {
        let lines: Vec<&str> = src.lines().collect();
        for (i, l) in lines.iter().enumerate() {
            let Some(tags) = l.strip_prefix("```ail ") else { continue };
            let words: Vec<&str> = tags.split_whitespace().collect();
            let named = words.iter().any(|w| w.strip_prefix("rules=").is_some_and(|ids| ids.split(',').any(|id| id == rule)));
            if !named {
                continue;
            }
            let body: Vec<&str> = lines[i + 1..].iter().take_while(|l| !l.starts_with("```")).copied().collect();
            let tool = words.iter().find_map(|w| w.strip_prefix("tool="));
            let blank = body.iter().all(|l| l.trim().is_empty());
            // A block with no code (`pkg=` or `fixture=`) shows its command alone.
            if blank && tool.is_none() {
                continue;
            }
            let mut out = String::new();
            if let Some(pkg) = words.iter().find_map(|w| w.strip_prefix("pkg=")) {
                let _ = writeln!(out, "# package {pkg}");
            }
            if let Some(cmd) = tool {
                let stdin = words.iter().find_map(|w| w.strip_prefix("stdin="));
                let _ = write!(out, "$ ail {}", cmd.replace(',', " "));
                if let Some(name) = stdin {
                    let _ = write!(out, " < {name}");
                }
                out.push('\n');
            }
            if !blank {
                // Four lines at most: a tagged block is often a whole program (ail explain stays short).
                out.push_str(&body.iter().take(4).copied().collect::<Vec<_>>().join("\n"));
            }
            return Some(out.trim_end().to_string());
        }
    }
    None
}

/// `[label](target)` as `label`: a link target costs tokens and an agent can't
/// follow it from the terminal (explain.cover.4, ADR 0336).
fn strip_links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        let link = after.find("](").and_then(|c| {
            let label = &after[..c];
            let end = after[c..].find(')')?;
            (!label.contains(['[', '\n'])).then_some((label, c + end))
        });
        match link {
            Some((label, end)) => {
                out.push_str(&rest[..open]);
                out.push_str(label);
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[..open + 1]);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
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
    let file = e.file.strip_prefix("docs/spec/").unwrap_or(&e.file);
    let mut s = format!("{} ({file}:{})\n{}\n", e.rule, e.line, shorten(&e.text));
    // A worked example (rejected and repaired) is printed whole: the repair
    // may be on any line. A plain example is cut.
    for (label, code, cut) in
        [("example", &e.example, true), ("rejected example", &e.rejected, false), ("repaired", &e.fixed, false)]
    {
        if let Some(x) = code {
            let _ = write!(s, "\n{label}:\n");
            let keep = if cut { EXAMPLE_LINES } else { usize::MAX };
            for l in x.lines().take(keep) {
                let _ = writeln!(s, "  {l}");
            }
            if x.lines().count() > keep {
                s.push_str("  ...\n");
            }
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
    let _ = write!(s, ",\"fix\":{}", fixable(&e.rule));
    if let Some(x) = &e.example {
        let _ = write!(s, ",\"example\":{}", json_str(x));
    }
    for (key, code) in [("rejected", &e.rejected), ("fixed", &e.fixed)] {
        if let Some(x) = code {
            let _ = write!(s, ",\"{key}\":{}", json_str(x));
        }
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
    fn a_rule_without_an_example_in_its_section_gets_its_tagged_block() {
        let e = explain("api.refs.1").unwrap();
        let example = e.example.unwrap();
        assert!(example.starts_with("$ ail api --refs"), "{example}");
        let diff = explain("diff.out.1").unwrap().example.unwrap();
        assert!(diff.starts_with("# package diff_pair\n$ ail diff old/a.ail new/a.ail"), "{diff}");
    }

    #[test]
    fn rule_ids_are_recognised() {
        assert!(is_rule_id("expr.hole.1"));
        assert!(!is_rule_id("expr.hole"));
        assert!(!is_rule_id("Expr.hole.1"));
        assert!(explain("no.such.rule").is_none());
        assert!(explain(UNSUPPORTED).is_some());
    }
}
