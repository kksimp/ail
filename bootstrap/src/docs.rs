//! Documentation comments: attach each run of `##` lines to the
//! declaration on the line right after it (lex.comment.2, api.doc.1-2).
//!
//! The lexer drops comments, so this reads the source text again rather
//! than threading comments through the token stream.

use std::collections::HashMap;

use crate::ast::{Program, is_prelude};
use crate::diag::Pos;

/// The doc comment ending on the line before each line, by that line's number.
/// A blank line or a plain `#` comment ends a run (api.doc.1).
pub fn by_line(src: &str) -> HashMap<usize, String> {
    let mut docs = HashMap::new();
    let mut run: Vec<&str> = Vec::new();
    for (i, raw) in src.split('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw).trim_start_matches([' ', '\t']);
        if let Some(text) = line.strip_prefix("##") {
            run.push(text.strip_prefix(' ').unwrap_or(text).trim_end());
            continue;
        }
        if !run.is_empty() && !line.is_empty() && !line.starts_with('#') {
            docs.insert(i + 1, run.join("\n"));
        }
        run.clear();
    }
    docs
}

/// Fill the `doc` of every declaration in the program (not the prelude)
/// from the source text it was parsed from (api.doc.2).
pub fn attach(prog: &mut Program, src: &str) {
    attach_lines(prog, &by_line(src));
}

/// Like `attach` for docs already keyed by program line, as in a package of
/// several files (api.card.1).
pub fn attach_lines(prog: &mut Program, docs: &HashMap<usize, String>) {
    let doc = |pos: Pos| if is_prelude(pos) { None } else { docs.get(&pos.line).cloned() };
    for f in &mut prog.fns {
        f.doc = doc(f.pos);
    }
    for s in &mut prog.structs {
        s.doc = doc(s.pos);
        for f in &mut s.fields {
            f.doc = doc(f.pos);
        }
    }
    for e in &mut prog.enums {
        e.doc = doc(e.pos);
        for v in &mut e.variants {
            v.doc = doc(v.pos);
            for f in &mut v.fields {
                // A variant's fields share its line; only a field on a line
                // of its own could carry a doc of its own.
                if f.pos.line != v.pos.line {
                    f.doc = doc(f.pos);
                }
            }
        }
    }
    for t in &mut prog.traits {
        t.doc = doc(t.pos);
    }
    for c in &mut prog.consts {
        c.doc = doc(c.pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_attach_to_the_next_line_only() {
        let docs = by_line("## one\n##  two\nfn f()\n\n## lost\n\nfn g()\n    ## inner\n    x int\n");
        assert_eq!(docs.get(&3).map(String::as_str), Some("one\n two"));
        assert_eq!(docs.get(&7), None);
        assert_eq!(docs.get(&9).map(String::as_str), Some("inner"));
    }

    #[test]
    fn a_plain_comment_breaks_the_run() {
        let docs = by_line("## doc\n# note\nfn f()\n");
        assert!(docs.is_empty());
    }
}
