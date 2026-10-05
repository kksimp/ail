//! Lexer: source text to tokens, including INDENT/DEDENT (ADR 0033).
//!
//! Spec rules enforced here: lex.block.1-4, lex.stmt.1-3, lex.kw.1,
//! lex.comment.1-3, lex.str.1-5, lex.num.1-3. Constructs the bootstrap doesn't
//! implement yet produce a `bootstrap.unsupported` diagnostic instead of a
//! wrong answer.

use crate::diag::{Diagnostic, Pos};

pub const UNSUPPORTED: &str = "bootstrap.unsupported";

/// Reserved words (lex.kw.1, ADR 0114). `c` is deliberately absent (lex.kw.2).
pub const KEYWORDS: &[&str] = &[
    "and", "any", "as", "break", "continue", "defer", "elif", "else", "enum", "extern", "false", "fn", "for", "if",
    "impl", "in", "match", "move", "mut", "none", "not", "or", "pub", "ref", "return", "self", "spawn", "struct",
    "trait", "true", "unique", "unsafe", "use", "while",
];

/// Symbols, longest first so that `..=` wins over `..` and `.`.
const SYMBOLS: &[&str] = &[
    "..=", "..", "->", "=>", ":=", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "&&", "||", ".", ":", ",",
    "(", ")", "[", "]", "{", "}", "=", "<", ">", "+", "-", "*", "/", "%", "!", "?", "@",
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
    Int(i64),
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub pos: Pos,
}

/// The indentation unit is exactly four spaces (lex.block.2, ADR 0040).
const UNIT: usize = 4;

pub fn lex(src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lx = Lexer { tokens: Vec::new(), diags: Vec::new(), open: Vec::new(), in_interp: 0 };
    let mut indents = vec![0usize];
    let mut continuing = false; // previous line ended in a way that continues the statement
    let mut last_line = 1;

    for (i, raw) in src.split('\n').enumerate() {
        let line_no = i + 1;
        last_line = line_no;
        let line: Vec<char> = raw.strip_suffix('\r').unwrap_or(raw).chars().collect();

        let lead = line.iter().take_while(|c| **c == ' ' || **c == '\t').count();
        let rest_is_blank = line[lead..].is_empty() || line[lead] == '#';
        if rest_is_blank {
            continue; // blank and comment-only lines don't affect blocks (lex.block note)
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

        let before = lx.tokens.len();
        lx.lex_line(&line, lead, line_no);
        let produced = lx.tokens.len() > before;

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
                    lx.push(Tok::Newline, Pos { line: line_no, col: line.len() + 1 });
                }
                continuing = false;
            }
        }
    }

    let end = Pos { line: last_line, col: 1 };
    if let Some((c, pos)) = lx.open.last().copied() {
        lx.diags.push(Diagnostic::new("lex.stmt.2", pos, format!("this `{c}` is never closed")));
        lx.push(Tok::Newline, end);
    }
    while indents.len() > 1 {
        indents.pop();
        lx.push(Tok::Dedent, end);
    }
    lx.push(Tok::Eof, end);
    (lx.tokens, lx.diags)
}

struct Lexer {
    tokens: Vec<Token>,
    diags: Vec<Diagnostic>,
    /// Unclosed `(`, `[` and `{`; newlines inside them don't end statements.
    open: Vec<(char, Pos)>,
    /// Depth of string interpolation being lexed.
    in_interp: usize,
}

impl Lexer {
    fn push(&mut self, tok: Tok, pos: Pos) {
        self.tokens.push(Token { tok, pos });
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

    /// Width of leading whitespace, reporting tabs and partial units.
    fn indent_width(&mut self, lead: &[char], line_no: usize) -> usize {
        if let Some(i) = lead.iter().position(|c| *c == '\t') {
            self.diags.push(
                Diagnostic::new("lex.block.4", Pos { line: line_no, col: i + 1 }, "tab character in indentation")
                    .help("indent with 4 spaces per level"),
            );
        }
        let width: usize = lead.iter().map(|c| if *c == '\t' { UNIT } else { 1 }).sum();
        if !width.is_multiple_of(UNIT) {
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

    fn lex_line(&mut self, line: &[char], start: usize, line_no: usize) {
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
                '#' => break, // comment to end of line (lex.comment.1, lex.comment.2)
                '"' => i = self.string(line, i, line_no),
                '\'' => {
                    self.unsupported(pos, "a character literal", "lex.str.4");
                    self.placeholder(pos);
                    i = line[i + 1..].iter().position(|c| *c == '\'').map_or(line.len(), |p| i + p + 2);
                }
                ';' => {
                    self.diags.push(
                        Diagnostic::new("lex.stmt.3", pos, "AIL has no `;`; a statement ends at the end of its line")
                            .help("remove the `;` and put the next statement on its own line"),
                    );
                    // Recover as if the line ended here, so the parser doesn't report it again.
                    if self.open.is_empty() && self.in_interp == 0 {
                        self.push(Tok::Newline, pos);
                    }
                    i += 1;
                }
                c if c.is_ascii_digit() => i = self.number(line, i, line_no),
                c if c.is_alphabetic() || c == '_' => {
                    let end = i + line[i..].iter().take_while(|c| c.is_alphanumeric() || **c == '_').count();
                    let word: String = line[i..end].iter().collect();
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
        if line.get(start + 1) == Some(&'"') && line.get(start + 2) == Some(&'"') {
            self.unsupported(pos, "a multi-line string", "lex.str.3");
            return line.len();
        }
        let mut parts = Vec::new();
        let mut value = String::new();
        let mut i = start + 1;
        while i < line.len() {
            let here = Pos { line: line_no, col: i + 1 };
            match line[i] {
                '"' => {
                    if parts.is_empty() {
                        self.push(Tok::Str(value), pos);
                    } else {
                        if !value.is_empty() {
                            parts.push(Part::Lit(value));
                        }
                        self.push(Tok::Interp(parts), pos);
                    }
                    return i + 1;
                }
                '\\' => {
                    let (ch, next) = self.escape(line, i, line_no);
                    if let Some(ch) = ch {
                        value.push(ch);
                    }
                    i = next;
                }
                '{' => {
                    // lex.str.2: `{expression}` interpolates.
                    let Some(close) = find_close(line, i) else {
                        self.diags.push(
                            Diagnostic::new("lex.str.2", here, "this `{` in a string is never closed")
                                .help("close it with `}`, or write `\\{` for a literal brace"),
                        );
                        return line.len();
                    };
                    let code = self.sub_lex(line, i + 1, close, line_no);
                    if code.len() == 1 {
                        self.diags.push(
                            Diagnostic::new("lex.str.2", here, "empty `{}` in a string")
                                .help("put an expression inside, or write `\\{` for a literal brace"),
                        );
                    } else {
                        if !value.is_empty() {
                            parts.push(Part::Lit(std::mem::take(&mut value)));
                        }
                        parts.push(Part::Code(code));
                    }
                    i = close + 1;
                }
                c => {
                    value.push(c);
                    i += 1;
                }
            }
        }
        self.diags
            .push(Diagnostic::new("lex.str.1", pos, "unterminated string literal").help("close the string with `\"`"));
        line.len()
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
            return match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                Some(c) if !hex.is_empty() => (Some(c), next),
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
                .help("valid escapes: \\n \\t \\r \\0 \\\\ \\\" \\{ \\u{hex}"),
        );
        (None, (i + 2).min(line.len()))
    }

    /// Integer literal (lex.num.1, lex.num.3); floats aren't supported yet.
    fn number(&mut self, line: &[char], start: usize, line_no: usize) -> usize {
        let pos = Pos { line: line_no, col: start + 1 };
        let (radix, digits_start) = match (line[start], line.get(start + 1)) {
            ('0', Some('x')) => (16, start + 2),
            ('0', Some('b')) => (2, start + 2),
            ('0', Some('o')) => (8, start + 2),
            _ => (10, start),
        };
        let end =
            digits_start + line[digits_start..].iter().take_while(|c| c.is_ascii_alphanumeric() || **c == '_').count();
        let is_float =
            radix == 10 && line.get(end) == Some(&'.') && line.get(end + 1).is_some_and(|c| c.is_ascii_digit());
        if is_float {
            self.unsupported(pos, "a floating-point literal", "lex.num.2");
            self.placeholder(pos);
            let frac = end + 1 + line[end + 1..].iter().take_while(|c| c.is_ascii_alphanumeric()).count();
            return frac;
        }
        let digits: String = line[digits_start..end].iter().filter(|c| **c != '_').collect();
        if radix == 10 && digits.contains(['e', 'E']) {
            self.unsupported(pos, "a floating-point literal", "lex.num.2");
            self.placeholder(pos);
            return end;
        }
        let text: String = line[start..end].iter().collect();
        match i64::from_str_radix(&digits, radix) {
            Ok(v) => self.push(Tok::Int(v), pos),
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

/// Index of the `}` matching the `{` at `line[open]`, skipping nested strings.
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
        assert_eq!(rules("1.5"), vec![UNSUPPORTED]);
        assert_eq!(rules("f(1"), vec!["lex.stmt.2"]);
        assert_eq!(rules("a; b"), vec!["lex.stmt.3"]);
    }
}
