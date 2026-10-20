//! Doc examples as tests: each ```` ```ail ```` block in a `##` doc comment is
//! compiled by `ail test` as a test of its own, in the scope of the file
//! (test.doc.1-3).
//!
//! The blocks become `@test` functions appended to the source, so the
//! checker and interpreter treat them like any other test. Their positions
//! are mapped back to the lines of the doc comment.

use std::collections::HashMap;

use crate::ast::{Program, is_prelude};

/// One example.
pub struct DocTest {
    /// `doc:Type.method` (test.doc.3).
    pub name: String,
    /// The generated function's name.
    pub fn_name: String,
    /// The lines `# => text` asked for, when there are any (test.doc.2).
    pub expected: Option<Vec<String>>,
    /// The code with its line in the original file.
    code: Vec<(usize, String)>,
    /// The line of the opening fence.
    fence: usize,
}

/// The name of each documented declaration, by its line.
fn declared(prog: &Program) -> HashMap<usize, String> {
    let mut names = HashMap::new();
    let mut add = |pos: crate::diag::Pos, name: String| {
        if !is_prelude(pos) {
            names.insert(pos.line, name);
        }
    };
    for f in &prog.fns {
        add(f.pos, f.full_name());
    }
    for s in &prog.structs {
        add(s.pos, s.name.clone());
    }
    for e in &prog.enums {
        add(e.pos, e.name.clone());
        for v in &e.variants {
            add(v.pos, format!("{}.{}", e.name, v.name));
        }
    }
    for t in &prog.traits {
        add(t.pos, t.name.clone());
    }
    for c in &prog.consts {
        add(c.pos, c.name.clone());
    }
    names
}

/// The ```` ```ail ```` blocks of every doc comment, in source order
/// (test.doc.1). A block tagged anything but exactly `ail` (`ail fragment`,
/// `text`) is not run.
pub fn collect(src: &str, prog: &Program) -> Vec<DocTest> {
    let names = declared(prog);
    let mut found: Vec<DocTest> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut run: Vec<(usize, String)> = Vec::new();
    for (i, raw) in src.split('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw).trim_start_matches([' ', '\t']);
        if let Some(text) = line.strip_prefix("##") {
            run.push((i + 1, text.strip_prefix(' ').unwrap_or(text).trim_end().to_string()));
            continue;
        }
        if !run.is_empty()
            && !line.is_empty()
            && !line.starts_with('#')
            && let Some(name) = names.get(&(i + 1))
        {
            for block in blocks(&run) {
                let n = seen.entry(name.clone()).or_insert(0);
                *n += 1;
                let label = if *n == 1 { format!("doc:{name}") } else { format!("doc:{name}#{n}") };
                found.push(DocTest { name: label, ..block });
            }
        }
        run.clear();
    }
    for (k, t) in found.iter_mut().enumerate() {
        t.fn_name = format!("doc_example_{}", k + 1);
    }
    found
}

/// The runnable blocks of one doc comment.
fn blocks(run: &[(usize, String)]) -> Vec<DocTest> {
    let mut out = Vec::new();
    let mut open: Option<(usize, Vec<(usize, String)>)> = None;
    for (line, text) in run {
        match (&mut open, text.trim()) {
            (None, "```ail") => open = Some((*line, Vec::new())),
            (None, _) => {}
            (Some(_), "```") => {
                let (fence, code) = open.take().expect("open block");
                if code.iter().any(|(_, c)| !c.trim().is_empty()) {
                    let expected: Vec<String> = code
                        .iter()
                        .filter_map(|(_, c)| c.trim().strip_prefix("# =>").map(|t| t.strip_prefix(' ').unwrap_or(t).to_string()))
                        .collect();
                    out.push(DocTest {
                        name: String::new(),
                        fn_name: String::new(),
                        expected: if expected.is_empty() { None } else { Some(expected) },
                        code,
                        fence,
                    });
                }
            }
            (Some((_, code)), _) => code.push((*line, text.clone())),
        }
    }
    out
}

/// The source with each example appended as a test, and the original line
/// of every appended line (test.doc.1).
pub fn augment(src: &str, tests: &[DocTest]) -> (String, Vec<usize>) {
    let mut out = src.to_string();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    let base = out.matches('\n').count();
    let mut origin = Vec::new();
    for t in tests {
        out.push_str(&format!("@test fn {}()\n", t.fn_name));
        origin.push(t.fence);
        for (line, code) in &t.code {
            if code.trim().is_empty() {
                out.push('\n');
            } else {
                out.push_str("    ");
                out.push_str(code);
                out.push('\n');
            }
            origin.push(*line);
        }
    }
    // `base` lines of the original, then one entry per appended line.
    let mut map = vec![0; base];
    map.extend(origin);
    (out, map)
}

/// The text a doc example's output must equal, or the failure (test.doc.2).
pub fn check_output(t: &DocTest, output: &str) -> Option<String> {
    let want = t.expected.as_ref()?;
    let got: Vec<&str> = output.lines().collect();
    if got.iter().map(|s| s.trim_end()).eq(want.iter().map(|s| s.trim_end())) {
        return None;
    }
    Some(format!("trap[test.doc.2]: output differs: expected {:?}, found {:?}", want, got))
}

impl DocTest {
    /// The lines of its `# =>` expectations (snap.update.4).
    pub fn expected_lines(&self) -> Vec<usize> {
        self.code.iter().filter(|(_, c)| c.trim().starts_with("# =>")).map(|(l, _)| *l).collect()
    }

    /// Where a failing example is reported: its opening fence.
    pub fn line(&self) -> usize {
        self.fence
    }
}
