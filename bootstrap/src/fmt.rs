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
        Formatter { lines, logicals, comments: comments.to_vec(), out: Vec::new() }
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
        let mut lines = layout(&l.toks, l.depth);
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
    if prefix(before, a) {
        return false;
    }
    if b.is("(") || b.is("[") {
        return !a.ends_operand();
    }
    true
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
    for (i, t) in toks.iter().enumerate() {
        if t.is(",") && toks.get(i + 1).is_some_and(FTok::closes) {
            continue;
        }
        if let Some(p) = prev
            && spaced(before, p, t)
        {
            out.push(' ');
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
    let mut stack: Vec<bool> = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if t.closes() {
            let list = stack.pop().unwrap_or(false);
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
            stack.push(list_bracket(toks, i));
        }
        out.push(t.clone());
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
    for (from, to) in elements(toks, group.open, group.close) {
        let mut rendered = layout(&toks[from..to], depth + 1);
        rendered.last_mut().unwrap().push(',');
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

/// The comma-separated elements between `toks[open]` and `toks[close]`, as ranges.
fn elements(toks: &[FTok], open: usize, close: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut level = 0;
    let mut start = open + 1;
    for (i, t) in toks.iter().enumerate().take(close).skip(open + 1) {
        if t.opens() {
            level += 1;
        } else if t.closes() {
            level -= 1;
        } else if level == 0 && t.is(",") {
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
