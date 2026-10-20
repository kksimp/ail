//! `ail fmt`: the canonical layout of AIL source (ADR 0169).
//!
//! The style is non-normative and is described, rule by rule, in
//! `docs/design/formatting.md`; the `fmt.*` IDs cited here are that file's.
//! The formatter works on the lexer's token stream, with the comment side
//! table from `lexer::lex_with_comments`. It only changes whitespace, line
//! breaks and trailing commas, and refuses to write output whose token stream
//! differs from the input's (fmt.safe).

use crate::diag::{Diagnostic, Pos};
use crate::lexer::{self, Comment, Tok, Token};
use crate::parser;

/// The line-length limit (fmt.wrap).
pub const WIDTH: usize = 100;
const UNIT: usize = 4;
/// A comment line before the first statement that keeps the file as written (fmt.off).
pub const OFF: &str = "# ail fmt: off";

/// Why a file wasn't formatted.
#[derive(Debug)]
pub enum Refusal {
    /// The file doesn't lex or parse; these are the diagnostics.
    Rejected(Vec<Diagnostic>),
    /// A formatter bug: the output wouldn't mean the same as the input (fmt.safe).
    Internal(String),
}

/// The canonical form of `src`.
pub fn format(src: &str) -> Result<String, Refusal> {
    let (tokens, mut diags, comments) = lexer::lex_with_comments(src);
    if diags.is_empty() {
        diags = parser::parse(tokens.clone()).1;
    }
    if !diags.is_empty() {
        return Err(Refusal::Rejected(diags));
    }
    // fmt.off: a lexer test whose layout is the point opts out on its first lines.
    let first_code = tokens.iter().find(|t| real(&t.tok)).map_or(usize::MAX, |t| t.pos.line);
    if comments.iter().any(|c| c.pos.line < first_code && c.text.starts_with(OFF)) {
        return Ok(src.to_string());
    }
    let out = Formatter::new(src, &tokens, &comments).run();
    // fmt.safe: the same tokens, apart from trailing commas, and the same comments.
    let (again, again_diags, again_comments) = lexer::lex_with_comments(&out);
    if !again_diags.is_empty() {
        return Err(Refusal::Internal(format!("the output doesn't lex: {}", again_diags[0].message)));
    }
    if significant(&tokens) != significant(&again) {
        return Err(Refusal::Internal("the output's tokens differ from the input's".into()));
    }
    let texts = |cs: &[Comment]| cs.iter().map(|c| c.text.clone()).collect::<Vec<_>>();
    if texts(&comments) != texts(&again_comments) {
        return Err(Refusal::Internal("the output's comments differ from the input's".into()));
    }
    Ok(out)
}

/// The tokens that carry meaning: everything but a comma before a closing bracket (ADR 0118).
/// Positions are left out, including those inside interpolated strings.
fn significant(tokens: &[Token]) -> Vec<String> {
    attributes_canonical(significant_raw(tokens))
}

/// Attributes in a fixed order, on one line (fmt.attr): a run of `@name(...)`
/// groups at the start of a statement is sorted by name, and a newline between
/// the run and the item is dropped, so layout and order don't count as a change.
fn attributes_canonical(toks: Vec<String>) -> Vec<String> {
    let at = "Sym(\"@\")";
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    let mut starts = true;
    while i < toks.len() {
        if starts && toks[i] == at {
            let mut groups: Vec<Vec<String>> = Vec::new();
            while i < toks.len() && toks[i] == at {
                let from = i;
                i += 2; // `@` and the name
                if toks.get(i).is_some_and(|t| t == "Sym(\"(\")") {
                    let mut depth = 0;
                    while i < toks.len() {
                        match toks[i].as_str() {
                            "Sym(\"(\")" => depth += 1,
                            "Sym(\")\")" => depth -= 1,
                            _ => {}
                        }
                        i += 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
                groups.push(toks[from..i.min(toks.len())].to_vec());
                if toks.get(i).is_some_and(|t| t == "Newline") && toks.get(i + 1).is_some_and(|t| t == at || t == "Kw(\"fn\")") {
                    i += 1;
                }
            }
            groups.sort_by(|a, b| a.get(1).cmp(&b.get(1)));
            out.extend(groups.into_iter().flatten());
            starts = false;
            continue;
        }
        starts = matches!(toks[i].as_str(), "Newline" | "Indent" | "Dedent");
        out.push(toks[i].clone());
        i += 1;
    }
    out
}

fn significant_raw(tokens: &[Token]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        let closes = matches!(tokens.get(i + 1).map(|t| &t.tok), Some(Tok::Sym(")" | "]" | "}")));
        if t.tok == Tok::Sym(",") && closes {
            continue;
        }
        out.push(match &t.tok {
            Tok::Interp(parts) => {
                let parts: Vec<String> = parts
                    .iter()
                    .map(|p| match p {
                        lexer::Part::Lit(s) => format!("{s:?}"),
                        lexer::Part::Code(code) => format!("{{{}}}", significant(code).join(" ")),
                        lexer::Part::Fmt(code, spec) => format!("{{{}:{spec:?}}}", significant(code).join(" ")),
                    })
                    .collect();
                format!("interp({})", parts.join(","))
            }
            tok => format!("{tok:?}"),
        });
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Word,
    Kw(&'static str),
    Sym(&'static str),
    Lit,
}

/// A token with its exact source text.
#[derive(Clone, Debug)]
struct FTok {
    text: String,
    kind: Kind,
    line: usize,
    end_line: usize,
    /// Whitespace followed it in the source (decides prefix `!`, fmt.space).
    space_after: bool,
}

impl FTok {
    fn is(&self, sym: &str) -> bool {
        matches!(self.kind, Kind::Sym(s) if s == sym)
    }
    fn opens(&self) -> bool {
        matches!(self.kind, Kind::Sym("(" | "[" | "{"))
    }
    fn closes(&self) -> bool {
        matches!(self.kind, Kind::Sym(")" | "]" | "}"))
    }
    /// Ends an operand, so a following `(` or `[` is a call or an index.
    fn ends_operand(&self) -> bool {
        matches!(self.kind, Kind::Word | Kind::Lit | Kind::Kw("self") | Kind::Sym(")" | "]" | "}" | "?"))
    }
    fn comma() -> FTok {
        FTok { text: ",".into(), kind: Kind::Sym(","), line: 0, end_line: 0, space_after: true }
    }
}

/// A statement: the tokens between two `Newline`s (lex.stmt.1-2).
struct Logical {
    depth: usize,
    toks: Vec<FTok>,
    opens_block: bool,
    start: usize,
    end: usize,
}

struct Formatter<'a> {
    lines: Vec<&'a str>,
    logicals: Vec<Logical>,
    comments: Vec<Comment>,
    out: Vec<String>,
}

impl<'a> Formatter<'a> {
    fn new(src: &'a str, tokens: &[Token], comments: &[Comment]) -> Self {
        let lines: Vec<&str> = src.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();
        let chars: Vec<Vec<char>> = lines.iter().map(|l| l.chars().collect()).collect();
        // Every token and comment start, in source order, bounds the text before it.
        let mut starts: Vec<Pos> = tokens.iter().filter(|t| real(&t.tok)).map(|t| t.pos).collect();
        starts.extend(comments.iter().map(|c| c.pos));
        starts.sort_by_key(|p| (p.line, p.col));
        starts.dedup();

        let mut logicals = Vec::new();
        let mut depth = 0;
        let mut current: Vec<FTok> = Vec::new();
        let mut cur_depth = 0;
        for (i, t) in tokens.iter().enumerate() {
            match &t.tok {
                Tok::Indent => depth += 1,
                Tok::Dedent => depth -= 1,
                Tok::Eof => {}
                Tok::Newline => {
                    if current.is_empty() {
                        continue;
                    }
                    let toks = std::mem::take(&mut current);
                    let opens_block = matches!(tokens.get(i + 1).map(|t| &t.tok), Some(Tok::Indent));
                    let (start, end) = (toks[0].line, toks.last().unwrap().end_line);
                    logicals.push(Logical { depth: cur_depth, toks, opens_block, start, end });
                }
                tok => {
                    if current.is_empty() {
                        cur_depth = depth;
                    }
                    let next = starts.partition_point(|p| (p.line, p.col) <= (t.pos.line, t.pos.col));
                    let (text, end_line, space_after) = slice(&chars, t.pos, starts.get(next).copied());
                    let kind = match tok {
                        Tok::Ident(_) => Kind::Word,
                        Tok::Kw(k) => Kind::Kw(k),
                        Tok::Sym(s) => Kind::Sym(s),
                        _ => Kind::Lit,
                    };
                    current.push(FTok { text, kind, line: t.pos.line, end_line, space_after });
                }
            }
        }
        // fmt.attr: an attributes-only line joins the item below it, unless a comment sits between.
        let mut merged: Vec<Logical> = Vec::new();
        for l in logicals {
            match merged.last_mut() {
                Some(prev)
                    if attributes_only(&prev.toks)
                        && prev.depth == l.depth
                        && !comments.iter().any(|c| c.pos.line >= prev.end && c.pos.line < l.start) =>
                {
                    prev.toks.extend(l.toks);
                    prev.end = l.end;
                    prev.opens_block = l.opens_block;
                }
                _ => merged.push(l),
            }
        }
        Formatter { lines, logicals: merged, comments: comments.to_vec(), out: Vec::new() }
    }

    fn run(mut self) -> String {
        // Sort comments into each statement's own (inside or after it) and the rest.
        let logicals = std::mem::take(&mut self.logicals);
        let mut owned: Vec<Vec<Comment>> = logicals.iter().map(|_| Vec::new()).collect();
        let mut free: Vec<Comment> = Vec::new();
        for c in &self.comments {
            match logicals.iter().position(|l| l.start <= c.pos.line && c.pos.line <= l.end) {
                Some(i) => owned[i].push(c.clone()),
                None => free.push(c.clone()),
            }
        }

        // Events in source order: statements and free comment lines.
        enum Ev {
            Code(usize),
            Note(Comment),
        }
        let mut events: Vec<(usize, Ev)> = logicals.iter().enumerate().map(|(i, l)| (l.start, Ev::Code(i))).collect();
        events.extend(free.into_iter().map(|c| (c.pos.line, Ev::Note(c))));
        events.sort_by_key(|(line, _)| *line);

        let first_line = |e: &Ev| match e {
            Ev::Code(i) => logicals[*i].start,
            Ev::Note(c) => c.pos.line,
        };
        let last_line = |e: &Ev| match e {
            Ev::Code(i) => logicals[*i].end,
            Ev::Note(c) => c.pos.line,
        };

        // For each event: the statement before it, and the statement it leads
        // up to through adjacent comment lines.
        let n = events.len();
        let gap = |k: usize| k > 0 && first_line(&events[k].1) > last_line(&events[k - 1].1) + 1;
        let mut prev = vec![None; n];
        let mut last = None;
        for (k, (_, e)) in events.iter().enumerate() {
            prev[k] = last;
            if let Ev::Code(i) = e {
                last = Some(*i);
            }
        }
        let mut lead = vec![None; n];
        let mut next = None;
        for k in (0..n).rev() {
            if let Ev::Code(i) = &events[k].1 {
                next = Some(*i);
            }
            lead[k] = next;
            if gap(k) {
                next = None;
            }
        }
        // fmt.comment: a comment line keeps its own depth, clamped to the
        // blocks around it; at the depth of the statement after it, it
        // belongs to that statement.
        let mut note_depth = vec![0; n];
        let mut attached = vec![false; n];
        for (k, (_, e)) in events.iter().enumerate() {
            if let Ev::Note(c) = e {
                let hi = prev[k].map_or(0, |p: usize| logicals[p].depth + logicals[p].opens_block as usize);
                let lo = lead[k].map_or(0, |i: usize| logicals[i].depth).min(hi);
                note_depth[k] = ((c.pos.col - 1) / UNIT).clamp(lo, hi);
                attached[k] = lead[k].is_some_and(|i| logicals[i].depth == note_depth[k]);
            }
        }

        let mut last_top_multi = false;
        let mut seen_top = false;
        for k in 0..n {
            let ev = &events[k].1;
            let starts_group = k == 0 || gap(k) || !attached[k - 1];
            let group_lead = match ev {
                Ev::Note(_) if !attached[k] => None,
                _ => lead[k],
            };
            let mut blank = gap(k);
            if starts_group && let Some(i) = group_lead {
                let l = &logicals[i];
                let first_in_block = prev[k].is_some_and(|p| logicals[p].depth < l.depth);
                if first_in_block {
                    blank = false; // fmt.blank: no blank line opens a block
                } else if l.depth == 0 && seen_top {
                    let multi = l.opens_block || self.render(l, &owned[i]).len() > 1;
                    blank = gap(k) || multi || last_top_multi; // fmt.blank: one between declarations
                }
            }
            if blank && !self.out.is_empty() {
                self.out.push(String::new());
            }
            match ev {
                Ev::Note(c) => self.out.push(format!("{}{}", indent(note_depth[k]), c.text)),
                Ev::Code(i) => {
                    let l = &logicals[*i];
                    let before = self.out.len();
                    let rendered = self.render(l, &owned[*i]);
                    self.out.extend(rendered);
                    let multi = l.opens_block || self.out.len() - before > 1;
                    if l.depth == 0 {
                        seen_top = true;
                        last_top_multi = multi;
                    } else {
                        last_top_multi = true; // inside a block: the declaration spans lines
                    }
                }
            }
        }
        let mut text = self.out.join("\n");
        text.push('\n');
        if text.trim().is_empty() {
            return String::new();
        }
        text
    }

    /// The lines of one statement.
    fn render(&self, l: &Logical, comments: &[Comment]) -> Vec<String> {
        // A multi-line string's lines are its content: leave the statement as written.
        if l.start != l.end && l.toks.iter().any(|t| t.line != t.end_line) {
            return self.lines[l.start - 1..l.end].iter().map(|s| s.to_string()).collect();
        }
        let trailing = comments.iter().find(|c| c.pos.line == l.end);
        let interior = comments.iter().any(|c| c.pos.line < l.end);
        if interior {
            return self.render_as_broken(l, comments);
        }
        let mut lines = layout_item(&l.toks, l.depth);
        if let Some(c) = trailing {
            let last = lines.last_mut().unwrap();
            last.push(' ');
            last.push_str(&c.text);
        }
        lines
    }

    /// fmt.wrap: a statement with comments inside keeps its line breaks; each
    /// line is indented by its bracket depth, and a closing bracket on its own
    /// line gets a trailing comma before it (fmt.comma).
    fn render_as_broken(&self, l: &Logical, comments: &[Comment]) -> Vec<String> {
        let toks = with_trailing_commas(&l.toks);
        let mut lines = Vec::new();
        let mut open = 0usize;
        let mut i = 0;
        let mut ci = 0;
        for line_no in l.start..=l.end {
            let from = i;
            while i < toks.len() && (toks[i].line == line_no || toks[i].line == 0 && i > from) {
                i += 1;
            }
            let row = &toks[from..i];
            let depth = if line_no == l.start {
                l.depth
            } else if row.first().is_some_and(FTok::closes) {
                l.depth + open.saturating_sub(1)
            } else {
                l.depth + open.max(1)
            };
            let mut text = String::new();
            if !row.is_empty() {
                text = format!("{}{}", indent(depth), flat(row));
            }
            for t in row {
                if t.opens() {
                    open += 1;
                } else if t.closes() {
                    open = open.saturating_sub(1);
                }
            }
            while ci < comments.len() && comments[ci].pos.line < line_no {
                ci += 1;
            }
            if let Some(c) = comments.get(ci).filter(|c| c.pos.line == line_no) {
                if text.is_empty() {
                    text = format!("{}{}", indent(l.depth + open.max(1)), c.text);
                } else {
                    text.push(' ');
                    text.push_str(&c.text);
                }
            }
            if !text.is_empty() {
                lines.push(text);
            }
        }
        lines
    }
}

fn real(tok: &Tok) -> bool {
    !matches!(tok, Tok::Newline | Tok::Indent | Tok::Dedent | Tok::Eof)
}

fn indent(depth: usize) -> String {
    " ".repeat(depth * UNIT)
}

/// The source text from `from` up to `to` (or the end of the source), without
/// trailing whitespace; the line it ends on; and whether whitespace followed it.
fn slice(lines: &[Vec<char>], from: Pos, to: Option<Pos>) -> (String, usize, bool) {
    let mut text = String::new();
    let (mut line, mut col) = (from.line, from.col);
    loop {
        if to.is_some_and(|t| (line, col) >= (t.line, t.col)) || line > lines.len() {
            break;
        }
        match lines[line - 1].get(col - 1) {
            Some(c) => {
                text.push(*c);
                col += 1;
            }
            None => {
                text.push('\n');
                line += 1;
                col = 1;
            }
        }
    }
    let trimmed = text.trim_end();
    let space_after = trimmed.len() < text.len();
    let end_line = from.line + trimmed.matches('\n').count();
    (trimmed.to_string(), end_line, space_after)
}

/// Whether a space separates `a` and `b`; `before` precedes `a` (fmt.space).
fn spaced(before: Option<&FTok>, a: &FTok, b: &FTok) -> bool {
    if matches!(b.kind, Kind::Sym("," | ")" | "]" | "}" | "." | "?" | ":")) {
        return false;
    }
    if matches!(a.kind, Kind::Sym("(" | "[" | "{" | "." | "@")) {
        return false;
    }
    if a.is("..") || a.is("..=") || b.is("..") || b.is("..=") {
        return false;
    }
    // `|x| key` (expr.key.1): a `|` after `(`, `,` or `:` opens the parameter.
    if a.is("|") && before.is_some_and(opens_key) {
        return false;
    }
    if prefix(before, a) {
        return false;
    }
    if b.is("(") || b.is("[") {
        return !a.ends_operand();
    }
    true
}

/// A `|` after this token opens a key function's parameter (expr.key.1).
fn opens_key(t: &FTok) -> bool {
    t.is("(") || t.is(",") || t.is(":")
}

/// `a` is a prefix operator: `-x`, `!T`.
fn prefix(before: Option<&FTok>, a: &FTok) -> bool {
    match a.kind {
        Kind::Sym("-") => before.is_none_or(|t| !t.ends_operand()),
        // `!T` and `T ! E` differ only by position in a type; the source's spacing says which.
        Kind::Sym("!") => !a.space_after && before.is_none_or(|t| !t.ends_operand() || t.is(")")),
        _ => false,
    }
}

/// Tokens on one line, without trailing commas (fmt.comma).
fn flat(toks: &[FTok]) -> String {
    let mut out = String::new();
    let mut prev: Option<&FTok> = None;
    let mut before: Option<&FTok> = None;
    // The opening `|` of a key function `|x|` (expr.key.1), whose closing `|` is unspaced.
    let mut open_bar = None;
    for (i, t) in toks.iter().enumerate() {
        if t.is(",") && toks.get(i + 1).is_some_and(FTok::closes) {
            continue;
        }
        let closing_bar = t.is("|") && i >= 2 && open_bar == Some(i - 2);
        if let Some(p) = prev
            && !closing_bar
            && spaced(before, p, t)
        {
            out.push(' ');
        }
        if t.is("|") && prev.is_some_and(opens_key) {
            open_bar = Some(i);
        }
        out.push_str(&t.text);
        before = prev;
        prev = Some(t);
    }
    out
}

/// A bracket whose contents are a comma-separated list that may be split
/// one element per line: arguments, parameters, list and map literals.
/// A parenthesised expression, an index and type arguments are not.
fn list_bracket(toks: &[FTok], i: usize) -> bool {
    let after_operand = i > 0 && toks[i - 1].ends_operand();
    match toks[i].kind {
        Kind::Sym("(") => after_operand,
        Kind::Sym("[") => !after_operand,
        Kind::Sym("{") => true,
        _ => false,
    }
}

/// Insert `,` before each closing list bracket that starts a line (fmt.comma).
fn with_trailing_commas(toks: &[FTok]) -> Vec<FTok> {
    let mut out: Vec<FTok> = Vec::new();
    // Per open bracket: whether it holds a list, and whether a comprehension
    // (no trailing comma, fmt.comp).
    let mut stack: Vec<(bool, bool)> = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if t.kind == Kind::Kw("for") {
            if let Some(top) = stack.last_mut() {
                top.1 = true;
            }
        }
        if t.closes() {
            let (list, comp) = stack.pop().unwrap_or((false, false));
            let list = list && !comp;
            let starts_line = i > 0 && toks[i - 1].end_line < t.line;
            let prev = out.last();
            if list && starts_line && prev.is_some_and(|p| !p.is(",") && !p.opens()) {
                let mut comma = FTok::comma();
                comma.line = prev.unwrap().end_line;
                comma.end_line = comma.line;
                out.push(comma);
            }
        }
        if t.opens() {
            stack.push((list_bracket(toks, i), false));
        }
        out.push(t.clone());
    }
    out
}

/// The end of the attribute starting at `toks[i]` (`@name` or `@name(...)`).
fn attribute_end(toks: &[FTok], i: usize) -> usize {
    let mut j = i + 2;
    if toks.get(j).is_some_and(|t| t.is("(")) {
        let mut level = 0;
        while j < toks.len() {
            if toks[j].opens() {
                level += 1;
            } else if toks[j].closes() {
                level -= 1;
            }
            j += 1;
            if level == 0 {
                break;
            }
        }
    }
    j.min(toks.len())
}

/// The statement is only attributes: the item they apply to is on a later line.
fn attributes_only(toks: &[FTok]) -> bool {
    let mut i = 0;
    while i < toks.len() && toks[i].is("@") {
        i = attribute_end(toks, i);
    }
    i == toks.len() && !toks.is_empty()
}

/// fmt.attr: attributes sorted by name; on the item's line if it all fits,
/// otherwise one per line above the item.
fn layout_item(toks: &[FTok], depth: usize) -> Vec<String> {
    let mut groups: Vec<&[FTok]> = Vec::new();
    let mut i = 0;
    while i < toks.len() && toks[i].is("@") {
        let end = attribute_end(toks, i);
        groups.push(&toks[i..end]);
        i = end;
    }
    if groups.is_empty() || i == toks.len() {
        return layout(toks, depth);
    }
    groups.sort_by(|a, b| a.get(1).map(|t| &t.text).cmp(&b.get(1).map(|t| &t.text)));
    let rest = &toks[i..];
    let all: Vec<FTok> = groups.iter().flat_map(|g| g.iter().cloned()).chain(rest.iter().cloned()).collect();
    let one = format!("{}{}", indent(depth), flat(&all));
    if one.chars().count() <= WIDTH {
        return vec![one];
    }
    let mut lines: Vec<String> = groups.iter().map(|g| format!("{}{}", indent(depth), flat(g))).collect();
    lines.extend(layout(rest, depth));
    lines
}

/// The start of each `.method(` call at the top level: a chained call.
fn chain_dots(toks: &[FTok]) -> Vec<usize> {
    let mut level = 0;
    let mut out: Vec<usize> = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        // An operator after the first call ends the chain's shape: `a.b() or c.d()` isn't a chain.
        if level == 0 && !out.is_empty() && !(t.is(".") || t.is("?") || t.kind == Kind::Word || t.opens() || t.closes()) {
            return Vec::new();
        }
        if t.opens() {
            level += 1;
        } else if t.closes() {
            level -= 1;
        } else if level == 0
            && t.is(".")
            && i > 0
            && toks.get(i + 1).is_some_and(|n| n.kind == Kind::Word)
            && toks.get(i + 2).is_some_and(|n| n.is("("))
        {
            out.push(i);
        }
    }
    out
}

/// fmt.wrap: one line if it fits in `WIDTH`. Otherwise the longest
/// top-level list is split one element per line, each with a trailing comma
/// (fmt.comma). When that list's last element ends in a list that holds most
/// of its text, the split moves into that inner list instead ("hugging":
/// `f(a, [` ... `])`). Elements that are still too long split the same way.
fn layout(toks: &[FTok], depth: usize) -> Vec<String> {
    let one = format!("{}{}", indent(depth), flat(toks));
    if one.chars().count() <= WIDTH {
        return vec![one];
    }
    // fmt.chain: two or more chained calls split one `.method(...)` per line.
    let dots = chain_dots(toks);
    if dots.len() >= 2 {
        let mut lines = layout(&toks[..dots[0]], depth);
        for (k, &from) in dots.iter().enumerate() {
            let to = dots.get(k + 1).copied().unwrap_or(toks.len());
            lines.extend(layout(&toks[from..to], depth + 1));
        }
        return lines;
    }
    let lines = layout_lists(toks, depth, one);
    if lines.iter().any(|l| l.chars().count() > WIDTH)
        && let Some(split) = layout_operators(toks, depth)
    {
        return split;
    }
    lines
}

/// The binary-operator precedence of `toks[i]` for splitting, or `None` (fmt.op).
/// `<` and `>` are left out: they also bracket type arguments.
fn split_prec(toks: &[FTok], i: usize) -> Option<u8> {
    let t = &toks[i];
    let binary = i > 0 && toks[i - 1].ends_operand();
    match t.kind {
        Kind::Kw("or") => Some(1),
        Kind::Kw("and") => Some(2),
        Kind::Sym("==" | "!=" | "<=" | ">=") => Some(3),
        Kind::Sym("+" | "-") if binary => Some(4),
        Kind::Sym("*" | "/" | "%") if binary => Some(5),
        _ => None,
    }
}

/// fmt.op: a line still too long is split before each top-level operator of the
/// lowest precedence, the operator first on the continuation line and one indent
/// deeper (lex.stmt.2 (5)). `-` ends its line instead, because a line may not start with it.
fn layout_operators(toks: &[FTok], depth: usize) -> Option<Vec<String>> {
    let mut level = 0;
    let mut points: Vec<(usize, u8)> = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if t.opens() {
            level += 1;
        } else if t.closes() {
            level -= 1;
        } else if level == 0 {
            if t.is("|") {
                return None; // a closure body runs to the end of the expression
            }
            if let Some(p) = split_prec(toks, i) {
                points.push((i, p));
            }
        }
    }
    let lowest = points.iter().map(|&(_, p)| p).min()?;
    let cuts: Vec<usize> = points
        .iter()
        .filter(|&&(_, p)| p == lowest)
        .map(|&(i, _)| i)
        .filter(|&i| i > 0 && i + 1 < toks.len())
        .collect();
    if cuts.is_empty() {
        return None;
    }
    let mut lines: Vec<String> = Vec::new();
    let mut from = 0;
    for &cut in &cuts {
        let minus = toks[cut].is("-");
        let end = if minus { cut + 1 } else { cut };
        lines.extend(layout(&toks[from..end], if from == 0 { depth } else { depth + 1 }));
        from = end;
    }
    lines.extend(layout(&toks[from..], depth + 1));
    Some(lines)
}

/// fmt.wrap's list splitting, for a line that has no chain (see `layout`).
fn layout_lists(toks: &[FTok], depth: usize, one: String) -> Vec<String> {
    let Some(mut group) = groups(toks, 0, toks.len()).into_iter().rev().max_by_key(|g| g.len) else {
        return vec![one]; // nothing to split: leave it long
    };
    loop {
        let elements = elements(toks, group.open, group.close);
        let Some(&(from, to)) = elements.last() else { break };
        let inner = groups(toks, from, to).into_iter().find(|g| g.close + 1 == to);
        let Some(inner) = inner else { break };
        let head = format!("{}{}", indent(depth), flat(&toks[..=inner.open]));
        if inner.len * 2 < group.len || head.chars().count() > WIDTH {
            break;
        }
        group = inner;
    }
    let mut lines = vec![format!("{}{}", indent(depth), flat(&toks[..=group.open]))];
    let comp = comprehension(toks, group.open, group.close);
    for (from, to) in elements(toks, group.open, group.close) {
        let mut rendered = layout(&toks[from..to], depth + 1);
        if !comp {
            rendered.last_mut().unwrap().push(',');
        }
        lines.extend(rendered);
    }
    lines.push(format!("{}{}", indent(depth), flat(&toks[group.close..])));
    lines
}

/// A bracket pair holding a list that may be split (see `list_bracket`).
#[derive(Clone, Copy)]
struct Group {
    open: usize,
    close: usize,
    /// The length of its contents.
    len: usize,
}

/// The splittable groups at the top level of `toks[from..to]`. A group of
/// one element with no list inside it gains nothing from a split and is left out.
fn groups(toks: &[FTok], from: usize, to: usize) -> Vec<Group> {
    let mut out = Vec::new();
    let mut level = 0;
    let mut open = 0;
    for i in from..to {
        if toks[i].opens() {
            if level == 0 {
                open = i;
            }
            level += 1;
        } else if toks[i].closes() {
            level -= 1;
            if level == 0 && i > open + 1 && list_bracket(toks, open) {
                let parts = elements(toks, open, i);
                let nested = parts.iter().any(|&(a, b)| !groups(toks, a, b).is_empty());
                if parts.len() > 1 || nested {
                    let len = toks[open + 1..i].iter().map(|t| t.text.chars().count() + 1).sum();
                    out.push(Group { open, close: i, len });
                }
            }
        }
    }
    out
}

/// Whether the brackets `toks[open]` and `toks[close]` hold a comprehension:
/// a `for` at their top level (expr.comp.1).
fn comprehension(toks: &[FTok], open: usize, close: usize) -> bool {
    let mut level = 0;
    for t in &toks[open + 1..close] {
        if t.opens() {
            level += 1;
        } else if t.closes() {
            level -= 1;
        } else if level == 0 && t.kind == Kind::Kw("for") {
            return true;
        }
    }
    false
}

/// The comma-separated elements between `toks[open]` and `toks[close]`, as
/// ranges. A comprehension's parts are its element and each `for` clause
/// (fmt.comp).
fn elements(toks: &[FTok], open: usize, close: usize) -> Vec<(usize, usize)> {
    let split_at_for = comprehension(toks, open, close);
    let mut out = Vec::new();
    let mut level = 0;
    let mut start = open + 1;
    for (i, t) in toks.iter().enumerate().take(close).skip(open + 1) {
        if t.opens() {
            level += 1;
        } else if t.closes() {
            level -= 1;
        } else if level == 0 && split_at_for && t.kind == Kind::Kw("for") {
            out.push((start, i));
            start = i;
        } else if level == 0 && !split_at_for && t.is(",") {
            out.push((start, i));
            start = i + 1;
        }
    }
    if start < close {
        out.push((start, close));
    }
    out
}

/// `ail fmt [--check | --stdout] PATH...`. Exit status: 0 done (or already
/// canonical), 1 `--check` found a file that would change, 2 a file couldn't
/// be read or formatted (ADR 0170).
pub fn command(args: &[&str], json: bool) -> std::process::ExitCode {
    use std::process::ExitCode;
    let check = args.contains(&"--check");
    let stdout = args.contains(&"--stdout");
    let paths: Vec<&str> = args.iter().copied().filter(|a| !a.starts_with("--")).collect();
    if let Some(bad) = args.iter().find(|a| a.starts_with("--") && !matches!(**a, "--check" | "--stdout")) {
        eprintln!("error: unknown option {bad} for `ail fmt`");
        return ExitCode::from(2);
    }
    if paths.is_empty() || (check && stdout) {
        eprintln!("usage: ail fmt [--check | --stdout] PATH...");
        return ExitCode::from(2);
    }
    let mut files = Vec::new();
    for p in &paths {
        let path = std::path::Path::new(p);
        if path.is_dir() {
            collect(path, &mut files);
        } else {
            files.push((p.to_string(), true));
        }
    }
    let (mut failed, mut changed, mut skipped) = (false, false, 0);
    for (file, named) in files {
        let src = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: can't read {file}: {e}");
                failed = true;
                continue;
            }
        };
        let out = match format(&src) {
            Ok(out) => out,
            Err(Refusal::Rejected(mut diags)) => {
                // A file found in a directory that doesn't parse (a negative test) is skipped.
                if named {
                    eprintln!("error: {file} wasn't formatted because it has errors (fmt.safe)");
                    crate::diag::report(&mut diags, &file, &src, json);
                    failed = true;
                } else {
                    skipped += 1;
                }
                continue;
            }
            Err(Refusal::Internal(why)) => {
                eprintln!("{file}: internal error in the formatter (fmt.safe): {why}; the file is unchanged");
                failed = true;
                continue;
            }
        };
        if stdout {
            print!("{out}");
        } else if out != src {
            changed = true;
            if check {
                print!("{}", diff(&file, &src, &out));
            } else if let Err(e) = std::fs::write(&file, &out) {
                eprintln!("error: can't write {file}: {e}");
                failed = true;
            }
        }
    }
    if skipped > 0 {
        eprintln!("note: skipped {skipped} file(s) that have lexical or syntax errors");
    }
    if failed {
        ExitCode::from(2)
    } else if check && changed {
        eprintln!("help: run `ail fmt` on these files to rewrite them in canonical layout");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Every `.ail` file under `dir`, sorted, skipping `target` and hidden directories.
fn collect(dir: &std::path::Path, files: &mut Vec<(String, bool)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for p in paths {
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if p.is_dir() {
            if !name.starts_with('.') && name != "target" {
                collect(&p, files);
            }
        } else if p.extension().is_some_and(|x| x == "ail") {
            files.push((p.display().to_string(), false));
        }
    }
}

/// A unified diff of `old` against `new`, labelled with `path`; empty when equal.
pub fn diff(path: &str, old: &str, new: &str) -> String {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    if a == b && old.ends_with('\n') == new.ends_with('\n') {
        return String::new();
    }
    // Longest common subsequence, then hunks with three lines of context.
    let (n, m) = (a.len(), b.len());
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a[i] == b[j] { lcs[i + 1][j + 1] + 1 } else { lcs[i + 1][j].max(lcs[i][j + 1]) };
        }
    }
    let mut ops: Vec<(char, usize, usize)> = Vec::new(); // (' ' | '-' | '+', index in a, index in b)
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            ops.push((' ', i, j));
            i += 1;
            j += 1;
        } else if i < n && (j == m || lcs[i + 1][j] >= lcs[i][j + 1]) {
            ops.push(('-', i, j));
            i += 1;
        } else {
            ops.push(('+', i, j));
            j += 1;
        }
    }
    let mut out = format!("--- {path}\n+++ {path} (formatted)\n");
    let changed: Vec<usize> = (0..ops.len()).filter(|k| ops[*k].0 != ' ').collect();
    if changed.is_empty() {
        out.push_str("\\ the files differ only in the final newline\n");
        return out;
    }
    let mut k = 0;
    while k < changed.len() {
        let start = changed[k].saturating_sub(3);
        let mut end = changed[k] + 1;
        while k < changed.len() && changed[k] <= end + 5 {
            end = (changed[k] + 4).min(ops.len());
            k += 1;
        }
        let hunk = &ops[start..end];
        let a_len = hunk.iter().filter(|o| o.0 != '+').count();
        let b_len = hunk.iter().filter(|o| o.0 != '-').count();
        out.push_str(&format!("@@ -{},{} +{},{} @@\n", hunk[0].1 + 1, a_len, hunk[0].2 + 1, b_len));
        for (op, i, j) in hunk {
            let text = if *op == '+' { b[*j] } else { a[*i] };
            out.push_str(&format!("{op}{text}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fmt(src: &str) -> String {
        match format(src) {
            Ok(s) => s,
            Err(e) => panic!("not formatted: {e:?}"),
        }
    }

    fn sources() -> Vec<(String, String)> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut out = Vec::new();
        for dir in [root.join("../examples"), root.join("tests/cases")] {
            let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
            files.sort();
            for f in files.into_iter().filter(|p| p.extension().is_some_and(|x| x == "ail")) {
                out.push((f.display().to_string(), std::fs::read_to_string(&f).unwrap()));
            }
        }
        out
    }

    #[test]
    fn chained_calls_split_one_per_line() {
        let src = "fn main()\n    names := people.filter(|p| p.age > 18 and p.active).map(|p| p.name.to_upper()).sorted().join(\", \")\n    print(names)\n";
        let out = fmt(src);
        assert_eq!(
            out,
            "fn main()\n    names := people\n        .filter(|p| p.age > 18 and p.active)\n        .map(|p| p.name.to_upper())\n        .sorted()\n        .join(\", \")\n    print(names)\n"
        );
        assert_eq!(fmt(&out), out);
        // A chain that fits is joined onto one line.
        assert_eq!(fmt("fn main()\n    x := a\n        .b()\n        .c()\n"), "fn main()\n    x := a.b().c()\n");
    }

    #[test]
    fn long_conditions_split_before_the_lowest_operator() {
        let src = "fn main()\n    if alpha_value > 5 and beta_value > 5 and alpha_value + beta_value > 1000 or alpha_value == beta_value or beta_value == 77\n        print(1)\n";
        let out = fmt(src);
        assert_eq!(
            out,
            "fn main()\n    if alpha_value > 5 and beta_value > 5 and alpha_value + beta_value > 1000\n        or alpha_value == beta_value\n        or beta_value == 77\n        print(1)\n"
        );
        assert_eq!(fmt(&out), out);
        let sum = "fn f() int\n    alpha_value_long_name * beta_value_long_name + alpha_value_long_name * beta_value_long_name - gamma_value_long_name\n";
        let out = fmt(sum);
        assert!(out.contains("\n        + alpha_value_long_name * beta_value_long_name -\n"), "{out}");
        assert_eq!(fmt(&out), out);
        // Short enough: joined.
        assert_eq!(fmt("fn f() bool\n    a\n        and b\n"), "fn f() bool\n    a and b\n");
    }

    #[test]
    fn attributes_are_sorted_and_move_up_only_when_long() {
        assert_eq!(fmt("@test\nfn a()\n    assert(true)\n"), "@test fn a()\n    assert(true)\n");
        assert_eq!(fmt("@test @pure fn a()\n    assert(true)\n"), "@pure @test fn a()\n    assert(true)\n");
        let long = "@test @pure fn a_function_with_a_long_name(first_parameter int, second_parameter int, third_parameter int) int\n    first_parameter\n";
        let out = fmt(long);
        assert!(out.starts_with("@pure\n@test\nfn a_function_with_a_long_name("), "{out}");
        assert_eq!(fmt(&out), out);
    }

    #[test]
    fn idempotent_and_canonical_over_the_repository() {
        let mut formatted = 0;
        for (name, src) in sources() {
            match format(&src) {
                Ok(once) => {
                    let twice = fmt(&once);
                    assert_eq!(once, twice, "{name}: formatting is not idempotent");
                    // The committed examples and cases are canonical (`ail fmt --check` passes).
                    assert_eq!(src, once, "{name} is not canonical:\n{}", diff(&name, &src, &once));
                    formatted += 1;
                }
                Err(Refusal::Rejected(_)) => {} // a negative case that doesn't lex or parse
                Err(Refusal::Internal(e)) => panic!("{name}: {e}"),
            }
        }
        assert!(formatted > 50, "only {formatted} files formatted");
    }

    #[test]
    fn spacing_and_indentation() {
        assert_eq!(
            fmt("fn  main( )\n    x:=-1+ 2*3\n    print( x )\n"),
            "fn main()\n    x := -1 + 2 * 3\n    print(x)\n"
        );
        assert_eq!(fmt("fn f(a int,b int) !int\n    a\n"), "fn f(a int, b int) !int\n    a\n");
        assert_eq!(fmt("fn f() int ! E\n    1\n"), "fn f() int ! E\n    1\n");
        assert_eq!(fmt("fn f()\n    xs.sorted(by:| x |-x)\n"), "fn f()\n    xs.sorted(by: |x| -x)\n");
        assert_eq!(fmt("x := xs[1 ..]\ny := a [0]\n"), "x := xs[1..]\ny := a[0]\n");
    }

    #[test]
    fn blank_lines_between_declarations() {
        let src = "A := 1\nB := 2\nfn f()\n    x\n\n\n\n    y\nfn g()\n    z\n";
        assert_eq!(fmt(src), "A := 1\nB := 2\n\nfn f()\n    x\n\n    y\n\nfn g()\n    z\n");
    }

    #[test]
    fn comments_are_kept_on_their_lines() {
        let src = "# head\n\n# about f\nfn f()   # trailing\n    x  # also\n        # deep\nfn g()\n    y\n";
        assert_eq!(fmt(src), "# head\n\n# about f\nfn f() # trailing\n    x # also\n    # deep\n\nfn g()\n    y\n");
    }

    #[test]
    fn short_lists_join_and_long_lists_split() {
        assert_eq!(fmt("X := [\n    1,\n    2,\n]\n"), "X := [1, 2]\n");
        let long: Vec<String> = (0..30).map(|i| format!("\"item{i}\"")).collect();
        let out = fmt(&format!("X := [{}]\n", long.join(", ")));
        assert!(out.starts_with("X := [\n    \"item0\",\n"), "{out}");
        assert!(out.ends_with("    \"item29\",\n]\n"), "{out}");
    }

    #[test]
    fn comprehensions_split_at_for_without_commas() {
        assert_eq!(fmt("X := [\n    x\n    for x in xs\n]\n"), "X := [x for x in xs]\n");
        let long = "a_rather_long_name_for_the_list";
        let out = fmt(&format!("X := [Pos(c, r) for r in {long}.rows() if r > 0 for c in {long}.columns()]\n"));
        assert_eq!(
            out,
            format!("X := [\n    Pos(c, r)\n    for r in {long}.rows() if r > 0\n    for c in {long}.columns()\n]\n")
        );
    }

    #[test]
    fn comments_inside_a_list_keep_its_breaks() {
        let src = "X := [\n    1, # one\n    2\n]\n";
        assert_eq!(fmt(src), "X := [\n    1, # one\n    2,\n]\n");
    }

    #[test]
    fn rejects_what_does_not_parse() {
        assert!(matches!(format("fn f(\n"), Err(Refusal::Rejected(_))));
        assert!(matches!(format("fn f()\n   x\n"), Err(Refusal::Rejected(_))));
    }

    #[test]
    fn diff_is_unified() {
        let d = diff("a.ail", "x\ny\nz\n", "x\nY\nz\n");
        assert_eq!(d, "--- a.ail\n+++ a.ail (formatted)\n@@ -1,3 +1,3 @@\n x\n-y\n+Y\n z\n");
        assert_eq!(diff("a.ail", "x\n", "x\n"), "");
    }
}
