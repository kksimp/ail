//! Lexer: source text to tokens, including INDENT/DEDENT (ADR 0033).
//!
//! Spec rules enforced here: lex.block.1-4, lex.stmt.1-3, lex.kw.1,
//! lex.comment.1-3, lex.str.1-5, lex.num.1-3. Constructs the bootstrap doesn't
//! implement yet produce a `bootstrap.unsupported` diagnostic instead of a
//! wrong answer.

use crate::diag::{Diagnostic, Edit, Pos};

pub const UNSUPPORTED: &str = "bootstrap.unsupported";

/// Reserved words (lex.kw.1, ADR 0114, ADR 0120, ADR 0210). `c` is deliberately absent (lex.kw.2).
pub const KEYWORDS: &[&str] = &[
    "and", "any", "as", "break", "continue", "defer", "elif", "else", "ensures", "enum", "error", "extern", "false",
    "fn", "for", "if", "impl", "in", "invariant", "match", "move", "mut", "none", "not", "or", "par", "pub", "ref", "requires",
    "return", "self", "spawn", "struct", "todo", "trait", "true", "unique", "unsafe", "use", "while", "with",
];

/// Symbols, longest first so that `..=` wins over `..` and `.`.
pub const SYMBOLS: &[&str] = &[
    "..=", "..", "->", "=>", ":=", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "&&", "||", ".", ":", ",",
    "(", ")", "[", "]", "{", "}", "=", "<", ">", "+", "-", "*", "/", "%", "!", "?", "@", "|",
];

/// A line ending in one of these continues on the next line (lex.stmt.2).
const BINARY_SYMBOLS: &[&str] = &["+", "-", "*", "/", "%", "==", "!=", "<", "<=", ">", ">=", "..", "..="];

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Kw(&'static str),
    Sym(&'static str),
    Str(String),
    /// A string literal containing `{expression}` parts (lex.str.2).
    Interp(Vec<Part>),
    /// `'a'` (lex.str.4)
    Char(char),
    Int(i64),
    /// `9223372036854775808`, which fits only after a `-` (lex.num.3).
    IntMin,
    /// `1.5`, `1e3` (lex.num.2)
    Float(f64),
    Newline,
    Indent,
    Dedent,
    Eof,
}

/// A piece of an interpolated string: literal text, or the tokens of one
/// embedded expression (ending in `Eof`).
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    Lit(String),
    Code(Vec<Token>),
    /// `{expression:spec}` (lex.str.7): the expression's tokens and its
    /// format specification.
    Fmt(Vec<Token>, FormatSpec),
}

/// A format specification after the `:` in `{expression:spec}` (lex.str.7):
/// `[[fill]align][width][.precision][x]`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FormatSpec {
    pub fill: Option<char>,
    /// `<`, `>` or `^`.
    pub align: Option<char>,
    pub width: Option<usize>,
    /// Decimal places of a float (`.N`).
    pub precision: Option<usize>,
    /// `x`: lowercase hexadecimal for an int.
    pub hex: bool,
}

impl FormatSpec {
    /// Parse the text after the `:`; the error is the problem, for lex.str.7.
    pub fn parse(spec: &str) -> Result<FormatSpec, String> {
        let c: Vec<char> = spec.chars().collect();
        let is_align = |ch: char| matches!(ch, '<' | '>' | '^');
        let mut f = FormatSpec::default();
        let mut i = 0;
        if c.len() >= 2 && is_align(c[1]) {
            f.fill = Some(c[0]);
            f.align = Some(c[1]);
            i = 2;
        } else if !c.is_empty() && is_align(c[0]) {
            f.align = Some(c[0]);
            i = 1;
        }
        let number = |c: &[char], i: &mut usize| -> Result<Option<usize>, String> {
            let start = *i;
            while *i < c.len() && c[*i].is_ascii_digit() {
                *i += 1;
            }
            if start == *i {
                return Ok(None);
            }
            let text: String = c[start..*i].iter().collect();
            text.parse().map(Some).map_err(|_| format!("`{text}` is too large"))
        };
        if c.get(i) == Some(&'0') && c.get(i + 1).is_some_and(char::is_ascii_digit) {
            return Err("a width can't start with 0; write the fill and alignment, e.g. `0>4`".into());
        }
        f.width = number(&c, &mut i)?;
        if c.get(i) == Some(&'.') {
            i += 1;
            f.precision = Some(number(&c, &mut i)?.ok_or("`.` needs the number of decimal places")?);
        }
        if c.get(i) == Some(&'x') {
            f.hex = true;
            i += 1;
        }
        if i < c.len() {
            let rest: String = c[i..].iter().collect();
            return Err(format!("unexpected `{rest}`"));
        }
        if f.width.is_some_and(|w| w > MAX_FORMAT_WIDTH) {
            return Err(format!("a width is at most {MAX_FORMAT_WIDTH}"));
        }
        if f.precision.is_some_and(|p| p > MAX_FORMAT_WIDTH) {
            return Err(format!("decimal places are at most {MAX_FORMAT_WIDTH}"));
        }
        if f.hex && f.precision.is_some() {
            return Err("`x` and `.N` can't be combined".into());
        }
        Ok(f)
    }
}

/// Largest width or precision a format specification accepts (lex.str.7).
pub const MAX_FORMAT_WIDTH: usize = 1000;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub pos: Pos,
}

/// The indentation unit is exactly four spaces (lex.block.2, ADR 0040).
const UNIT: usize = 4;

pub fn lex(src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let (tokens, diags, _) = lex_with_comments(src);
    (tokens, diags)
}

/// A comment (lex.comment.1): `text` runs from the `#` to the end of the line.
/// The parser never sees comments; the formatter keeps them (ADR 0169).
#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    pub pos: Pos,
    pub text: String,
}

/// `lex`, plus a side table of every comment in source order. The token
/// stream is exactly the one `lex` returns.
pub fn lex_with_comments(src: &str) -> (Vec<Token>, Vec<Diagnostic>, Vec<Comment>) {
    let (tokens, diags, comments, unit) = lex_scaled(src, 1);
    match unit {
        // The file indents by 2 or 3 spaces (lex.block.2, ADR 0263): lex it
        // again at that unit, so one diagnostic covers the whole file.
        Some(g) => {
            let (tokens, diags, comments, _) = lex_scaled(src, g);
            (tokens, diags, comments)
        }
        None => (tokens, diags, comments),
    }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Lex with one indentation unit of `scale` spaces; the last result is the
/// unit to retry with when the file doesn't indent by 4.
fn lex_scaled(src: &str, scale: usize) -> (Vec<Token>, Vec<Diagnostic>, Vec<Comment>, Option<usize>) {
    let mut lx = Lexer {
        tokens: Vec::new(),
        diags: Vec::new(),
        open: Vec::new(),
        in_interp: 0,
        comments: Vec::new(),
        scale,
        logical: Vec::new(),
        tabs: Vec::new(),
    };
    let mut indents = vec![0usize];
    let mut continuing = false; // previous line ended in a way that continues the statement
    let lines: Vec<Vec<char>> =
        src.split('\n').map(|raw| raw.strip_suffix('\r').unwrap_or(raw).chars().collect()).collect();
    let mut last_line = 1;
    let mut li = 0;

    while li < lines.len() {
        let line_no = li + 1;
        last_line = line_no;
        let line = &lines[li];

        let lead = line.iter().take_while(|c| **c == ' ' || **c == '\t').count();
        let rest_is_blank = line[lead..].is_empty() || line[lead] == '#';
        if rest_is_blank {
            lx.comment(line, lead, line_no);
            li += 1;
            continue; // blank and comment-only lines don't affect blocks (lex.block note)
        }

        // lex.stmt.2 (4): a line starting with `.name` continues the previous statement
        // (a chained call); the statement's newline is taken back.
        if lx.open.is_empty()
            && !continuing
            && line[lead] == '.'
            && line.get(lead + 1).is_some_and(|c| c.is_alphabetic() || *c == '_')
            && matches!(lx.tokens.last().map(|t| &t.tok), Some(Tok::Newline))
            && lx.tokens.len() >= 2
            && !matches!(lx.tokens[lx.tokens.len() - 2].tok, Tok::Indent | Tok::Dedent)
        {
            lx.tokens.pop();
            continuing = true;
        }

        if lx.open.is_empty() && !continuing {
            let width = lx.indent_width(&line[..lead], line_no);
            let top = *indents.last().unwrap();
            let pos = Pos { line: line_no, col: 1 };
            if width > top {
                if width != top + UNIT {
                    lx.diags.push(
                        Diagnostic::new(
                            "lex.block.3",
                            Pos { line: line_no, col: width + 1 },
                            format!(
                                "indentation increases by {} spaces; a block is indented by exactly 4",
                                width - top
                            ),
                        )
                        .help(format!("indent this line by {} spaces", top + UNIT)),
                    );
                }
                indents.push(width);
                lx.push(Tok::Indent, pos);
            } else {
                while width < *indents.last().unwrap() {
                    indents.pop();
                    lx.push(Tok::Dedent, pos);
                }
                if width != *indents.last().unwrap() {
                    lx.diags.push(Diagnostic::new(
                        "lex.block.1",
                        Pos { line: line_no, col: width + 1 },
                        "this line's indentation doesn't match any enclosing block",
                    ));
                    indents.push(width);
                }
            }
        }

        // A multi-line string continues the line onto later lines (lex.str.3).
        let before = lx.tokens.len();
        let mut from = lead;
        while let Some((col, raw)) = lx.lex_line(&lines[li], from, li + 1) {
            (li, from) = lx.multiline(&lines, li, col, raw);
        }
        let produced = lx.tokens.len() > before;
        let end_col = lines[li].len() + 1;
        last_line = li + 1;

        if lx.open.is_empty() {
            // lex.stmt.2: a line ending in a comma or a binary operator continues.
            let continues = match lx.tokens.last().map(|t| &t.tok) {
                Some(Tok::Sym(s)) => *s == "," || BINARY_SYMBOLS.contains(s),
                Some(Tok::Kw(k)) => *k == "and" || *k == "or",
                _ => false,
            };
            if produced && continues {
                continuing = true;
            } else {
                if produced || continuing {
                    lx.push(Tok::Newline, Pos { line: li + 1, col: end_col });
                }
                continuing = false;
            }
        }
        li += 1;
    }

    let end = Pos { line: last_line, col: 1 };
    if let Some((c, pos)) = lx.open.last().copied() {
        let mut d = Diagnostic::new("lex.stmt.2", pos, format!("this `{c}` is never closed"));
        // Close the brackets where the line ends, when they all opened on it.
        if lx.open.iter().all(|(_, p)| p.line == pos.line) {
            let closers: String = lx.open.iter().rev().map(|(c, _)| match c {
                '(' => ')',
                '[' => ']',
                _ => '}',
            }).collect();
            d = d.fix("close the brackets at the end of the line", vec![Edit::at_line_end(pos, closers)]);
        }
        lx.diags.push(d);
        // Close them for the parser too, so one report isn't followed by another.
        for (c, _) in lx.open.clone().into_iter().rev() {
            let sym = match c {
                '(' => ")",
                '[' => "]",
                _ => "}",
            };
            lx.tokens.push(Token { tok: Tok::Sym(sym), pos: end });
        }
        lx.push(Tok::Newline, end);
    }
    while indents.len() > 1 {
        indents.pop();
        lx.push(Tok::Dedent, end);
    }
    lx.push(Tok::Eof, end);
    let unit = lx.indent_diags();
    (lx.tokens, lx.diags, lx.comments, unit)
}

struct Lexer {
    tokens: Vec<Token>,
    diags: Vec<Diagnostic>,
    /// Unclosed `(`, `[` and `{`; newlines inside them don't end statements.
    open: Vec<(char, Pos)>,
    /// Depth of string interpolation being lexed.
    in_interp: usize,
    /// Comments, for the formatter (`lex_with_comments`).
    comments: Vec<Comment>,
    /// Spaces per indentation level in this run (1 = the standard 4).
    scale: usize,
    /// Every statement-starting line: its number, the characters of leading
    /// whitespace and their width (a tab counts as one unit).
    logical: Vec<(usize, usize, usize)>,
    /// The part of `logical` indented with a tab.
    tabs: Vec<(usize, usize, usize)>,
}

impl Lexer {
    fn push(&mut self, tok: Tok, pos: Pos) {
        self.tokens.push(Token { tok, pos });
    }

    /// Record the comment at `line[at..]`, if there is one.
    fn comment(&mut self, line: &[char], at: usize, line_no: usize) {
        if line.get(at) == Some(&'#') {
            let text: String = line[at..].iter().collect();
            self.comments.push(Comment { pos: Pos { line: line_no, col: at + 1 }, text: text.trim_end().to_string() });
        }
    }

    /// Stand-in for a literal that was reported as an error, so the parser
    /// doesn't report the same mistake again.
    fn placeholder(&mut self, pos: Pos) {
        self.push(Tok::Int(0), pos);
    }

    fn unsupported(&mut self, pos: Pos, what: &str, rule: &str) {
        self.diags.push(Diagnostic::new(
            UNSUPPORTED,
            pos,
            format!("{what} is not supported by the bootstrap yet (spec {rule})"),
        ));
    }

    /// Width of leading whitespace in indentation units of four. Tabs and
    /// other units are collected and reported once per file (`indent_diags`).
    fn indent_width(&mut self, lead: &[char], line_no: usize) -> usize {
        let tabbed = lead.contains(&'\t');
        let width: usize = lead.iter().map(|c| if *c == '\t' { UNIT } else { 1 }).sum();
        self.logical.push((line_no, lead.len(), width));
        if tabbed {
            self.tabs.push((line_no, lead.len(), width));
        } else if self.scale > 1 {
            return width / self.scale * UNIT;
        }
        if !tabbed && !width.is_multiple_of(UNIT) && self.scale == 1 {
            self.diags.push(
                Diagnostic::new(
                    "lex.block.2",
                    Pos { line: line_no, col: 1 },
                    format!("indentation of {width} spaces is not a multiple of 4"),
                )
                .help(format!("use {} spaces", width / UNIT * UNIT)),
            );
        }
        width / UNIT * UNIT
    }

    /// The indentation unit of a file that doesn't use 4: the common divisor
    /// of its statements' indentation, when that is 2 or 3 spaces.
    fn unit_guess(&self) -> Option<usize> {
        let g = self.logical.iter().map(|x| x.2).filter(|w| *w > 0).fold(0, gcd);
        let odd = self.logical.iter().any(|x| !x.2.is_multiple_of(UNIT));
        // The first level is the unit itself: a 4-space file with one odd
        // line has 4 there.
        let first = self.logical.iter().map(|x| x.2).filter(|w| *w > 0).min();
        (matches!(g, 2 | 3) && odd && first == Some(g) && self.tabs.is_empty()).then_some(g)
    }

    /// The file-level indentation diagnostics (lex.block.2, lex.block.4):
    /// one per file, each with a fix that rewrites every statement's
    /// indentation. Returns the unit to re-lex with when the file doesn't
    /// indent by 4.
    fn indent_diags(&mut self) -> Option<usize> {
        if let Some(&(line, _, _)) = self.tabs.first() {
            let edits: Vec<Edit> = self
                .tabs
                .iter()
                .map(|&(l, n, w)| Edit::span(Pos { line: l, col: 1 }, Pos { line: l, col: n + 1 }, " ".repeat(w / UNIT * UNIT)))
                .collect();
            self.diags.push(
                Diagnostic::new("lex.block.4", Pos { line, col: 1 }, "tab character in indentation")
                    .help("indent with 4 spaces per level")
                    .fix("replace the tabs with 4 spaces per level", edits),
            );
        }
        if self.scale == 1 {
            // The standard unit: an odd line was reported on its own, unless
            // the whole file turned out to use 2 or 3 spaces.
            return self.unit_guess();
        }
        let g = self.scale;
        let first = self.logical.iter().find(|x| x.2 > 0)?.0;
        let edits: Vec<Edit> = self
            .logical
            .iter()
            .filter(|x| x.2 > 0)
            .map(|&(l, n, w)| Edit::span(Pos { line: l, col: 1 }, Pos { line: l, col: n + 1 }, " ".repeat(w / g * UNIT)))
            .collect();
        self.diags.push(
            Diagnostic::new("lex.block.2", Pos { line: first, col: 1 }, format!("this file indents by {g} spaces; a block is indented by exactly 4"))
                .help("use 4 spaces per level")
                .fix(format!("re-indent the file from {g} to 4 spaces per level"), edits),
        );
        None
    }

    /// Lex `line[start..]`. Stops at a multi-line string's opening `"""`
    /// (lex.str.3), returning its column and whether it is raw, so the caller
    /// can read the following lines.
    fn lex_line(&mut self, line: &[char], start: usize, line_no: usize) -> Option<(usize, bool)> {
        let mut i = start;
        while i < line.len() {
            let c = line[i];
            let pos = Pos { line: line_no, col: i + 1 };
            match c {
                ' ' | '\t' => i += 1,
                '#' if self.in_interp > 0 => {
                    self.diags.push(Diagnostic::new(
                        "lex.str.2",
                        pos,
                        "a comment can't appear inside `{...}` in a string",
                    ));
                    i += 1;
                }
                '#' => {
                    // comment to end of line (lex.comment.1, lex.comment.2)
                    self.comment(line, i, line_no);
                    break;
                }
                '"' if starts_triple(line, i) => {
                    if self.in_interp > 0 {
                        self.diags.push(Diagnostic::new(
                            "lex.str.3",
                            pos,
                            "a multi-line string can't appear inside `{...}` in a string",
                        ));
                        return None;
                    }
                    return Some((i, false));
                }
                '"' => i = self.string(line, i, line_no),
                // lex.str.6: `r"..."` and `r"""..."""` are raw strings.
                'r' if line.get(i + 1) == Some(&'"') => {
                    if starts_triple(line, i + 1) && self.in_interp == 0 {
                        return Some((i, true));
                    }
                    i = self.raw_string(line, i, line_no);
                }
                '\'' => i = self.char_literal(line, i, line_no),
                ';' => {
                    let rest = line[i + 1..].iter().position(|c| *c != ' ' && *c != '\t');
                    let edit = match rest {
                        None => Edit::trim_before(pos),
                        Some(k) if line[i + 1 + k] != '#' => {
                            let indent: String = line.iter().take_while(|c| **c == ' ').collect();
                            Edit::span(pos, Pos { line: line_no, col: i + 2 + k }, format!("\n{indent}"))
                        }
                        Some(_) => Edit::trim_before(pos),
                    };
                    self.diags.push(
                        Diagnostic::new("lex.stmt.3", pos, "AIL has no `;`; a statement ends at the end of its line")
                            .help("remove the `;` and put the next statement on its own line")
                            .fix("end the statement here, without the `;`", vec![edit]),
                    );
                    // Recover as if the line ended here, so the parser doesn't report it again.
                    if self.open.is_empty() && self.in_interp == 0 {
                        self.push(Tok::Newline, pos);
                    }
                    i += 1;
                }
                '/' if line.get(i + 1) == Some(&'/') => {
                    let first = line[..i].iter().all(|c| *c == ' ' || *c == '\t');
                    let mut d = Diagnostic::new(
                        "gram.foreign.1",
                        pos,
                        "`//` isn't a comment or an operator in AIL",
                    );
                    d = if first {
                        d.help("comments start with `#`").fix("write `#`", vec![Edit::span(pos, Pos { line: line_no, col: i + 3 }, "#")])
                    } else {
                        d.help("write `#` for a comment; `/` is integer division and rounds toward zero")
                    };
                    self.diags.push(d);
                    break;
                }
                '/' if line.get(i + 1) == Some(&'*') => {
                    self.diags.push(
                        Diagnostic::new("gram.foreign.1", pos, "`/*` doesn't start a comment in AIL")
                            .help("comments start with `#` and run to the end of the line"),
                    );
                    break;
                }
                '\\' if line[i + 1..].iter().all(|c| c.is_whitespace()) => {
                    self.diags.push(
                        Diagnostic::new("gram.foreign.1", pos, "AIL doesn't use `\\` to continue a line")
                            .help("end the line with an operator or a comma, or wrap the expression in parentheses")
                            .fix("remove the `\\`", vec![Edit::trim_before(pos)]),
                    );
                    i += 1;
                }
                '&' if line.get(i + 1) != Some(&'&') => {
                    self.diags.push(
                        Diagnostic::new("gram.foreign.1", pos, "AIL has no `&`")
                            .help("write `and` for logic; there are no references to take, values are passed by value and `mut` marks changes"),
                    );
                    i += 1;
                }
                c if c.is_ascii_digit() => i = self.number(line, i, line_no),
                c if c.is_alphabetic() || c == '_' => {
                    let end = i + line[i..].iter().take_while(|c| c.is_alphanumeric() || **c == '_').count();
                    let word: String = line[i..end].iter().collect();
                    if matches!(word.as_str(), "f" | "u") && line.get(end) == Some(&'"') {
                        self.diags.push(
                            Diagnostic::new("gram.foreign.1", pos, format!("`{word}\"...\"` isn't a string prefix in AIL"))
                                .help("every string interpolates `{expression}`; write the string without the prefix")
                                .fix(format!("remove the `{word}`"), vec![Edit::span(pos, Pos { line: line_no, col: i + 2 }, "")]),
                        );
                        i = end;
                        continue;
                    }
                    let tok = match KEYWORDS.iter().find(|k| **k == word) {
                        Some(k) => Tok::Kw(k),
                        None => Tok::Ident(word),
                    };
                    self.push(tok, pos);
                    i = end;
                }
                _ => {
                    let rest: String = line[i..line.len().min(i + 3)].iter().collect();
                    match SYMBOLS.iter().find(|s| rest.starts_with(**s)) {
                        Some(sym) => {
                            self.symbol(sym, pos);
                            i += sym.len();
                        }
                        None => {
                            self.unsupported(pos, &format!("the `{c}` symbol"), "chapter 05");
                            i += 1;
                        }
                    }
                }
            }
        }
        None
    }

    fn symbol(&mut self, sym: &'static str, pos: Pos) {
        match sym {
            "(" | "[" | "{" => self.open.push((sym.chars().next().unwrap(), pos)),
            ")" | "]" | "}" => {
                self.open.pop();
            }
            _ => {}
        }
        self.push(Tok::Sym(sym), pos);
    }

    /// String literal starting at `line[start] == '"'`; returns the index after it.
    fn string(&mut self, line: &[char], start: usize, line_no: usize) -> usize {
        let pos = Pos { line: line_no, col: start + 1 };
        let mut parts = Vec::new();
        let mut value = String::new();
        match self.content(line, start + 1, line.len(), line_no, true, &mut value, &mut parts) {
            Some(close) => {
                self.push(string_token(value, parts), pos);
                close + 1
            }
            None => {
                self.diags.push(
                    Diagnostic::new("lex.str.1", pos, "unterminated string literal").help("close the string with `\"`"),
                );
                self.forget_brackets(line_no);
                line.len()
            }
        }
    }

    /// After a string error, the brackets opened on its line don't continue
    /// the statement, so the error isn't followed by others about them.
    fn forget_brackets(&mut self, line_no: usize) {
        self.open.retain(|(_, p)| p.line != line_no);
    }

    /// `r"..."`: no escapes and no interpolation (lex.str.6). `line[start]` is the `r`.
    fn raw_string(&mut self, line: &[char], start: usize, line_no: usize) -> usize {
        let pos = Pos { line: line_no, col: start + 1 };
        match line[start + 2..].iter().position(|c| *c == '"') {
            Some(len) => {
                self.push(Tok::Str(line[start + 2..start + 2 + len].iter().collect()), pos);
                start + 3 + len
            }
            None => {
                self.diags.push(
                    Diagnostic::new("lex.str.1", pos, "unterminated string literal").help("close the string with `\"`"),
                );
                self.forget_brackets(line_no);
                line.len()
            }
        }
    }

    /// The text of a string in `line[from..to]`, with escapes and `{...}`
    /// interpolation (lex.str.1-2), appended to `value` and `parts`. When
    /// `quoted`, it stops at an unescaped `"` and returns its index.
    #[allow(clippy::too_many_arguments)]
    fn content(
        &mut self,
        line: &[char],
        from: usize,
        to: usize,
        line_no: usize,
        quoted: bool,
        value: &mut String,
        parts: &mut Vec<Part>,
    ) -> Option<usize> {
        let mut i = from;
        while i < to {
            let here = Pos { line: line_no, col: i + 1 };
            match line[i] {
                '"' if quoted => return Some(i),
                '\\' => {
                    let (ch, next) = self.escape(line, i, line_no);
                    if let Some(ch) = ch {
                        value.push(ch);
                    }
                    i = next;
                }
                '{' => {
                    // lex.str.2: `{expression}` interpolates.
                    let Some(close) = find_close(&line[..to], i) else {
                        self.diags.push(
                            Diagnostic::new("lex.str.2", here, "this `{` in a string is never closed")
                                .help("close it with `}`, or write `\\{` for a literal brace"),
                        );
                        self.forget_brackets(line_no);
                        return quoted.then_some(to);
                    };
                    let colon = spec_colon(&line[..close], i + 1);
                    let code = self.sub_lex(line, i + 1, colon.unwrap_or(close), line_no);
                    let mut spec = None;
                    if let Some(c) = colon {
                        let text: String = line[c + 1..close].iter().collect();
                        match FormatSpec::parse(&text) {
                            Ok(f) => spec = Some(f),
                            Err(why) => self.diags.push(
                                Diagnostic::new(
                                    "lex.str.7",
                                    Pos { line: line_no, col: c + 2 },
                                    format!("invalid format specification `{text}`: {why}"),
                                )
                                .help("a specification is `[[fill]align][width][.decimals][x]`, e.g. `{n:>8}`, `{r:.2}`, `{n:0>4x}`"),
                            ),
                        }
                    }
                    if code.len() == 1 {
                        self.diags.push(
                            Diagnostic::new("lex.str.2", here, "empty `{}` in a string")
                                .help("put an expression inside, or write `\\{` for a literal brace"),
                        );
                    } else {
                        if !value.is_empty() {
                            parts.push(Part::Lit(std::mem::take(value)));
                        }
                        match spec {
                            Some(f) => parts.push(Part::Fmt(code, f)),
                            None => parts.push(Part::Code(code)),
                        }
                    }
                    i = close + 1;
                }
                c => {
                    value.push(c);
                    i += 1;
                }
            }
        }
        None
    }

    /// A multi-line string whose opening `"""` (or `r"""`) is at
    /// `lines[first][col]` (lex.str.3, lex.str.6). Returns the line and
    /// column just after the closing `"""`.
    fn multiline(&mut self, lines: &[Vec<char>], first: usize, col: usize, raw: bool) -> (usize, usize) {
        let pos = Pos { line: first + 1, col: col + 1 };
        let open_end = col + if raw { 4 } else { 3 };
        let opening = &lines[first];
        // `"""text"""` on one line: no dedenting (lex.str.3).
        if let Some(k) = find_triple(&opening[open_end..], raw) {
            let close = open_end + k;
            let mut value = String::new();
            let mut parts = Vec::new();
            if raw {
                value.extend(&opening[open_end..close]);
            } else {
                self.content(opening, open_end, close, first + 1, false, &mut value, &mut parts);
            }
            self.push(string_token(value, parts), pos);
            return (first, close + 3);
        }
        if opening[open_end..].iter().any(|c| *c != ' ') {
            self.diags.push(
                Diagnostic::new(
                    "lex.str.3",
                    Pos { line: first + 1, col: open_end + 1 },
                    "text after the opening `\"\"\"`",
                )
                .help("start the string's text on the next line, or close it on this line"),
            );
        }
        // Find the closing `"""`.
        let mut li = first + 1;
        let close = loop {
            if li >= lines.len() {
                self.diags.push(
                    Diagnostic::new("lex.str.3", pos, "unterminated multi-line string").help("close it with `\"\"\"`"),
                );
                self.placeholder_str(pos);
                let last = lines.len() - 1;
                return (last, lines[last].len());
            }
            if let Some(k) = find_triple(&lines[li], raw) {
                break k;
            }
            li += 1;
        };
        let last = li;
        let closing = &lines[last];
        let alone = closing[..close].iter().all(|c| *c == ' ');
        // The common indentation of the text lines (and of the closing line when
        // it holds only the delimiter) is removed.
        let indent = |l: &[char]| l.iter().take_while(|c| **c == ' ').count();
        let blank = |l: &[char]| l.iter().all(|c| *c == ' ' || *c == '\t');
        // Only spaces indent a multi-line string's text (lex.str.3).
        for (k, whole) in lines.iter().enumerate().take(last + 1).skip(first + 1) {
            let line = if k == last { &whole[..close] } else { &whole[..] };
            let lead = line.iter().take_while(|c| **c == ' ' || **c == '\t').count();
            if (k == last || !blank(line)) && line[..lead].contains(&'\t') {
                self.diags.push(
                    Diagnostic::new(
                        "lex.str.3",
                        Pos { line: k + 1, col: 1 },
                        "tab in a multi-line string's indentation",
                    )
                    .help("indent with spaces; write `\\t` for a tab in the text"),
                );
            }
        }
        let mut common = (first + 1..last).filter(|&k| !blank(&lines[k])).map(|k| indent(&lines[k])).min();
        let closing_indent = if alone { close } else { indent(&closing[..close]) };
        common = Some(common.map_or(closing_indent, |c| c.min(closing_indent)));
        let common = common.unwrap_or(0);
        let mut value = String::new();
        let mut parts = Vec::new();
        let mut text_lines: Vec<(usize, usize, usize)> =
            (first + 1..last).map(|k| (k, common.min(lines[k].len()), lines[k].len())).collect();
        if !alone {
            text_lines.push((last, common.min(close), close));
        }
        let n = text_lines.len();
        for (j, (k, from, to)) in text_lines.into_iter().enumerate() {
            let line = &lines[k];
            if blank(&line[from..to]) && k != last {
                // a line of only whitespace is an empty line of text
            } else if raw {
                value.extend(&line[from..to]);
            } else {
                self.content(line, from, to, k + 1, false, &mut value, &mut parts);
            }
            // The text ends with its last line: a closing delimiter on a line
            // of its own adds no newline (lex.str.3).
            if j + 1 < n {
                value.push('\n');
            }
        }
        self.push(string_token(value, parts), pos);
        (last, close + 3)
    }

    /// Stand-in for a string literal that was reported as an error.
    fn placeholder_str(&mut self, pos: Pos) {
        self.push(Tok::Str(String::new()), pos);
    }

    /// `'a'` or `'\n'` (lex.str.4); returns the index after it.
    fn char_literal(&mut self, line: &[char], start: usize, line_no: usize) -> usize {
        let pos = Pos { line: line_no, col: start + 1 };
        let (ch, next) = match line.get(start + 1) {
            Some('\\') => self.escape(line, start + 1, line_no),
            Some('\'') | None => (None, start + 1),
            Some(c) => (Some(*c), start + 2),
        };
        if line.get(next) == Some(&'\'') && next > start + 1 {
            match ch {
                Some(c) => self.push(Tok::Char(c), pos),
                None => self.placeholder(pos),
            }
            return next + 1;
        }
        if line.get(start + 1..start + 3) == Some(&['\'', '\'']) {
            self.diags.push(
                Diagnostic::new("lex.str.4", pos, "a `'` in a character literal must be escaped").help("write '\\''"),
            );
            self.placeholder(pos);
            return start + 3;
        }
        let end = line[start + 1..].iter().position(|c| *c == '\'').map_or(line.len(), |p| start + p + 2);
        let mut d = Diagnostic::new("lex.str.4", pos, "a character literal holds exactly one character")
            .help("write a string in double quotes: \"...\"");
        let body = &line[start + 1..end.saturating_sub(1).max(start + 1)];
        if end <= line.len() && line.get(end - 1) == Some(&'\'') && !body.iter().any(|c| matches!(c, '"' | '\\' | '{')) {
            let close = Pos { line: line_no, col: end };
            d = d.fix(
                "write it as a string",
                vec![Edit::span(pos, Pos { line: line_no, col: start + 2 }, "\""), Edit::span(close, Pos { line: line_no, col: end + 1 }, "\"")],
            );
        }
        self.diags.push(d);
        self.placeholder(pos);
        end
    }

    /// Tokens of the expression in `line[start..end]`, keeping real columns.
    fn sub_lex(&mut self, line: &[char], start: usize, end: usize, line_no: usize) -> Vec<Token> {
        let saved_tokens = std::mem::take(&mut self.tokens);
        let saved_open = std::mem::take(&mut self.open);
        self.in_interp += 1;
        self.lex_line(&line[..end], start, line_no);
        self.in_interp -= 1;
        self.open = saved_open;
        let mut code = std::mem::replace(&mut self.tokens, saved_tokens);
        code.push(Token { tok: Tok::Eof, pos: Pos { line: line_no, col: end + 1 } });
        code
    }

    /// Escape sequence at `line[i] == '\\'` (lex.str.1). Returns the character and next index.
    fn escape(&mut self, line: &[char], i: usize, line_no: usize) -> (Option<char>, usize) {
        let pos = Pos { line: line_no, col: i + 1 };
        let simple = match line.get(i + 1) {
            Some('n') => Some('\n'),
            Some('t') => Some('\t'),
            Some('r') => Some('\r'),
            Some('0') => Some('\0'),
            Some('\\') => Some('\\'),
            Some('"') => Some('"'),
            Some('\'') => Some('\''),
            Some('{') => Some('{'),
            _ => None,
        };
        if let Some(c) = simple {
            return (Some(c), i + 2);
        }
        if line.get(i + 1) == Some(&'u')
            && line.get(i + 2) == Some(&'{')
            && let Some(close) = line[i + 3..].iter().position(|c| *c == '}')
        {
            let hex: String = line[i + 3..i + 3 + close].iter().collect();
            let next = i + 4 + close;
            let digits = !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit());
            return match u32::from_str_radix(&hex, 16).ok().filter(|_| digits).and_then(char::from_u32) {
                Some(c) => (Some(c), next),
                _ => {
                    self.diags.push(Diagnostic::new(
                        "lex.str.1",
                        pos,
                        format!("`\\u{{{hex}}}` is not a valid Unicode scalar value"),
                    ));
                    (None, next)
                }
            };
        }
        let shown: String = line.get(i + 1).map(|c| c.to_string()).unwrap_or_default();
        self.diags.push(
            Diagnostic::new("lex.str.1", pos, format!("unknown escape sequence `\\{shown}`"))
                .help("valid escapes: \\n \\t \\r \\0 \\\\ \\\" \\' \\{ \\u{hex}"),
        );
        (None, (i + 2).min(line.len()))
    }

    /// Integer or floating-point literal (lex.num.1-3).
    fn number(&mut self, line: &[char], start: usize, line_no: usize) -> usize {
        let pos = Pos { line: line_no, col: start + 1 };
        if let Some(end) = float_end(line, start) {
            let text: String = line[start..end].iter().filter(|c| **c != '_').collect();
            match text.parse::<f64>() {
                Ok(v) if v.is_finite() => self.push(Tok::Float(v), pos),
                _ => {
                    let shown: String = line[start..end].iter().collect();
                    self.diags.push(Diagnostic::new(
                        "lex.num.3",
                        pos,
                        format!("floating-point literal `{shown}` doesn't fit in `float`"),
                    ));
                    self.placeholder(pos);
                }
            }
            return end;
        }
        let (radix, digits_start) = match (line[start], line.get(start + 1)) {
            ('0', Some('x')) => (16, start + 2),
            ('0', Some('b')) => (2, start + 2),
            ('0', Some('o')) => (8, start + 2),
            _ => (10, start),
        };
        let end =
            digits_start + line[digits_start..].iter().take_while(|c| c.is_ascii_alphanumeric() || **c == '_').count();
        let digits: String = line[digits_start..end].iter().filter(|c| **c != '_').collect();
        let text: String = line[start..end].iter().collect();
        match i64::from_str_radix(&digits, radix) {
            Ok(v) => self.push(Tok::Int(v), pos),
            // lex.num.3: the magnitude of the minimum `int` is checked with its `-`.
            Err(_) if u64::from_str_radix(&digits, radix) == Ok(1 << 63) => self.push(Tok::IntMin, pos),
            Err(e) => {
                let overflow = matches!(e.kind(), std::num::IntErrorKind::PosOverflow);
                let (rule, msg) = if overflow {
                    ("lex.num.3", format!("integer literal `{text}` doesn't fit in `int` (64-bit signed)"))
                } else {
                    ("lex.num.1", format!("invalid integer literal `{text}`"))
                };
                self.diags.push(Diagnostic::new(rule, pos, msg));
                self.placeholder(pos);
            }
        }
        end
    }
}

/// The token for a string's text: plain, or with interpolated parts.
fn string_token(mut value: String, mut parts: Vec<Part>) -> Tok {
    if parts.is_empty() {
        return Tok::Str(value);
    }
    if !value.is_empty() {
        parts.push(Part::Lit(std::mem::take(&mut value)));
    }
    Tok::Interp(parts)
}

fn starts_triple(line: &[char], i: usize) -> bool {
    line.get(i..i + 3) == Some(&['"', '"', '"'])
}

/// Index of the closing `"""` of a multi-line string on `line`, skipping
/// escapes and interpolations unless the string is raw.
fn find_triple(line: &[char], raw: bool) -> Option<usize> {
    let mut i = 0;
    while i < line.len() {
        match line[i] {
            '\\' if !raw => i += 2,
            '{' if !raw => i = find_close(line, i).map_or(line.len(), |c| c + 1),
            // In a run of more than three quotes, the last three close it.
            '"' if starts_triple(line, i) => {
                let mut end = i;
                while line.get(end + 3) == Some(&'"') {
                    end += 1;
                }
                return Some(end);
            }
            _ => i += 1,
        }
    }
    None
}

/// The end of a decimal floating-point literal starting at `line[start]`,
/// or `None` if it's an integer (lex.num.2): digits, then `.` and digits,
/// an exponent, or both.
fn float_end(line: &[char], start: usize) -> Option<usize> {
    let digits = |from: usize| from + line[from..].iter().take_while(|c| c.is_ascii_digit() || **c == '_').count();
    if line[start] == '0' && line.get(start + 1).is_some_and(|c| matches!(c, 'x' | 'b' | 'o')) {
        return None;
    }
    let mut end = digits(start);
    let mut float = false;
    if line.get(end) == Some(&'.') && line.get(end + 1).is_some_and(|c| c.is_ascii_digit()) {
        end = digits(end + 1);
        float = true;
    }
    if matches!(line.get(end), Some('e' | 'E')) {
        let sign = usize::from(matches!(line.get(end + 1), Some('+' | '-')));
        if line.get(end + 1 + sign).is_some_and(|c| c.is_ascii_digit()) {
            end = digits(end + 1 + sign);
            float = true;
        }
    }
    float.then_some(end)
}

/// Index of the `}` matching the `{` at `line[open]`, skipping nested strings.
/// The `:` that starts a format specification (lex.str.7) in the
/// interpolation whose expression begins at `from`: the first one outside
/// brackets, strings and character literals.
fn spec_colon(line: &[char], from: usize) -> Option<usize> {
    let mut depth = 0;
    let mut i = from;
    while i < line.len() {
        match line[i] {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '"' => {
                i = skip_string(line, i)?;
                continue;
            }
            '\'' if line.get(i + 1) == Some(&'\\') => {
                if let Some(end) = line[i + 3..].iter().position(|c| *c == '\'') {
                    i += end + 4;
                    continue;
                }
            }
            '\'' if line.get(i + 2) == Some(&'\'') => {
                i += 3;
                continue;
            }
            ':' if depth == 0 && line.get(i + 1) != Some(&'=') => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

fn find_close(line: &[char], open: usize) -> Option<usize> {
    let mut depth = 0;
    let mut i = open;
    while i < line.len() {
        match line[i] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            '"' => {
                i = skip_string(line, i)?;
                continue;
            }
            // A character literal such as '{' or '\'' (lex.str.4).
            '\'' if line.get(i + 1) == Some(&'\\') => {
                if let Some(end) = line[i + 3..].iter().position(|c| *c == '\'') {
                    i += end + 4;
                    continue;
                }
            }
            '\'' if line.get(i + 2) == Some(&'\'') => {
                i += 3;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Index just after the string literal starting at `line[start] == '"'`.
fn skip_string(line: &[char], start: usize) -> Option<usize> {
    let mut i = start + 1;
    while i < line.len() {
        match line[i] {
            '\\' => i += 2,
            '"' => return Some(i + 1),
            '{' => i = find_close(line, i)? + 1,
            _ => i += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        let (t, d) = lex(src);
        assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
        t.into_iter().map(|t| t.tok).collect()
    }

    fn rules(src: &str) -> Vec<&'static str> {
        lex(src).1.into_iter().map(|d| d.rule).collect()
    }

    #[test]
    fn hello_world_tokens() {
        use Tok::*;
        assert_eq!(
            toks("fn main()\n    print(\"Hello, world!\")\n"),
            vec![
                Kw("fn"),
                Ident("main".into()),
                Sym("("),
                Sym(")"),
                Newline,
                Indent,
                Ident("print".into()),
                Sym("("),
                Str("Hello, world!".into()),
                Sym(")"),
                Newline,
                Dedent,
                Eof
            ]
        );
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        use Tok::*;
        assert_eq!(
            toks("# top\n\nfn f()  # trailing\n\n    ## doc\n    g()\n"),
            vec![
                Kw("fn"),
                Ident("f".into()),
                Sym("("),
                Sym(")"),
                Newline,
                Indent,
                Ident("g".into()),
                Sym("("),
                Sym(")"),
                Newline,
                Dedent,
                Eof
            ]
        );
    }

    #[test]
    fn newlines_inside_brackets_and_after_commas_and_operators_continue() {
        let t = toks(
            "fn f()\n    g(1,\n        2)\n    h(3,\n      4)\n    xs := [\n        1,\n    ]\n    n := 1 +\n        2\n",
        );
        let newlines = t.iter().filter(|t| **t == Tok::Newline).count();
        assert_eq!(newlines, 5); // header, g(...), h(...), xs := [...], n := 1 + 2
        assert_eq!(t.iter().filter(|t| **t == Tok::Indent).count(), 1);
    }

    #[test]
    fn symbols_take_the_longest_match() {
        use Tok::*;
        assert_eq!(
            toks("a..=b..c.d"),
            vec![
                Ident("a".into()),
                Sym("..="),
                Ident("b".into()),
                Sym(".."),
                Ident("c".into()),
                Sym("."),
                Ident("d".into()),
                Newline,
                Eof
            ]
        );
        assert_eq!(toks("x += 1 => y")[1], Sym("+="));
        assert_eq!(toks("1..2")[1], Sym(".."));
    }

    #[test]
    fn keywords_are_reserved_tokens() {
        assert_eq!(toks("if x")[0], Tok::Kw("if"));
        assert_eq!(toks("c")[0], Tok::Ident("c".into())); // lex.kw.2
    }

    #[test]
    fn escapes_and_unicode() {
        assert_eq!(toks(r#""a\tb\n\"\\\{\u{1F600}""#)[0], Tok::Str("a\tb\n\"\\{\u{1F600}".into()));
    }

    #[test]
    fn interpolation_splits_text_and_code() {
        let t = toks(r#""hi {name}, {f("x}")}!""#);
        let Tok::Interp(parts) = &t[0] else { panic!("{t:?}") };
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], Part::Lit("hi ".into()));
        let Part::Code(code) = &parts[1] else { panic!() };
        assert_eq!(code[0].tok, Tok::Ident("name".into()));
        assert_eq!(code[0].pos.col, 6);
        let Part::Code(code) = &parts[3] else { panic!() };
        assert_eq!(code[2].tok, Tok::Str("x}".into()));
        assert_eq!(parts[4], Part::Lit("!".into()));
    }

    #[test]
    fn integer_forms() {
        assert_eq!(
            toks("1_000 0xff 0b101 0o17"),
            vec![Tok::Int(1000), Tok::Int(255), Tok::Int(5), Tok::Int(15), Tok::Newline, Tok::Eof]
        );
    }

    #[test]
    fn floats_and_chars() {
        assert_eq!(
            toks("1.5 1e3 2.5e-3 1_000.25 7 1..2"),
            vec![
                Tok::Float(1.5),
                Tok::Float(1000.0),
                Tok::Float(0.0025),
                Tok::Float(1000.25),
                Tok::Int(7),
                Tok::Int(1),
                Tok::Sym(".."),
                Tok::Int(2),
                Tok::Newline,
                Tok::Eof
            ]
        );
        assert_eq!(
            toks(r"'a' '\n' '\'' '\u{e9}'")[..4],
            [Tok::Char('a'), Tok::Char('\n'), Tok::Char('\''), Tok::Char('é')]
        );
    }

    #[test]
    fn multi_line_and_raw_strings() {
        let src = "fn f()\n    s := \"\"\"\n        one\n          two {x}\n\n        three\n        \"\"\"\n    t := r\"a\\{b}\"\n";
        let t = toks(src);
        let Tok::Interp(parts) = &t[8] else { panic!("{t:?}") };
        assert_eq!(parts[0], Part::Lit("one\n  two ".into()));
        assert_eq!(parts[2], Part::Lit("\n\nthree".into()));
        assert!(t.contains(&Tok::Str("a\\{b}".into())));
        // The statement continues after the closing delimiter.
        let t = toks("f(\"\"\"\n    x\n    \"\"\", 1)\n");
        assert_eq!(t[2], Tok::Str("x".into()));
        assert_eq!(t[3], Tok::Sym(","));
        let raw = toks("r\"\"\"\n  a\\n{b}\n  \"\"\"\n");
        assert_eq!(raw[0], Tok::Str("a\\n{b}".into()));
        // One line, and quotes at the end of the text.
        assert_eq!(toks(r#"r"""say "hi"""""#)[0], Tok::Str("say \"hi\"".into()));
        assert_eq!(toks(r#""""a\tb""""#)[0], Tok::Str("a\tb".into()));
    }

    #[test]
    fn indentation_errors_cite_rules() {
        assert_eq!(rules("fn f()\n   g()\n"), vec!["lex.block.2"]);
        assert_eq!(rules("fn f()\n        g()\n"), vec!["lex.block.3"]);
        assert_eq!(rules("fn f()\n\tg()\n"), vec!["lex.block.4"]);
    }

    #[test]
    fn string_and_number_errors_cite_rules() {
        assert_eq!(rules("\"abc"), vec!["lex.str.1"]);
        assert_eq!(rules(r#""\q""#), vec!["lex.str.1"]);
        assert_eq!(rules("99999999999999999999"), vec!["lex.num.3"]);
        assert_eq!(rules("\"a {} b\""), vec!["lex.str.2"]);
        assert_eq!(rules("\"a {b\""), vec!["lex.str.2"]);
        assert_eq!(rules("1e999"), vec!["lex.num.3"]);
        assert_eq!(rules("'ab'"), vec!["lex.str.4"]);
        assert_eq!(rules("x := \"\"\"\n  a\n"), vec!["lex.str.3"]);
        assert_eq!(rules("f(1"), vec!["lex.stmt.2"]);
        assert_eq!(rules("a; b"), vec!["lex.stmt.3"]);
    }
}
