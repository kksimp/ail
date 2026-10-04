//! Lexer: source text to tokens, including INDENT/DEDENT (ADR 0033).
//!
//! Spec rules enforced here: lex.block.1-4, lex.stmt.1-3, lex.comment.1-3,
//! lex.str.1-5, lex.num.1-3. Constructs the bootstrap doesn't implement yet
//! produce a `bootstrap.unsupported` diagnostic instead of a wrong answer.

use crate::diag::{Diagnostic, Pos};

pub const UNSUPPORTED: &str = "bootstrap.unsupported";

#[derive(Clone, Debug, PartialEq)]
pub enum Tok {
    Ident(String),
    Fn,
    Str(String),
    Int(i64),
    LParen,
    RParen,
    Comma,
    ColonEq,
    /// `:` alone: only used to report Rust/Python-style `name: Type` precisely.
    Colon,
    /// `->`: only used to report a return arrow precisely (gram.fn.2).
    Arrow,
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    pub pos: Pos,
}

/// The indentation unit is exactly four spaces (lex.block.2, ADR 0040).
const UNIT: usize = 4;

pub fn lex(src: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lx = Lexer { tokens: Vec::new(), diags: Vec::new(), depth: 0 };
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

        if lx.depth == 0 && !continuing {
            let width = lx.indent_width(&line[..lead], line_no);
            let top = *indents.last().unwrap();
            let pos = Pos { line: line_no, col: 1 };
            if width > top {
                if width != top + UNIT {
                    lx.diags.push(
                        Diagnostic::new(
                            "lex.block.3",
                            Pos { line: line_no, col: width + 1 },
                            format!("indentation increases by {} spaces; a block is indented by exactly 4", width - top),
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

        if lx.depth == 0 {
            let ends_with_comma = matches!(lx.tokens.last(), Some(Token { tok: Tok::Comma, .. }));
            if ends_with_comma {
                continuing = true; // lex.stmt.2: a line ending in a comma continues
            } else {
                if produced || continuing {
                    lx.push(Tok::Newline, Pos { line: line_no, col: line.len() + 1 });
                }
                continuing = false;
            }
        }
    }

    let end = Pos { line: last_line, col: 1 };
    if lx.depth > 0 {
        lx.diags.push(Diagnostic::new("lex.stmt.2", end, "unclosed `(` at end of file"));
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
    depth: usize,
}

impl Lexer {
    fn push(&mut self, tok: Tok, pos: Pos) {
        self.tokens.push(Token { tok, pos });
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
        if width % UNIT != 0 {
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
                ' ' => i += 1,
                '\t' => i += 1,
                '#' => break, // comment to end of line (lex.comment.1, lex.comment.2)
                '(' => {
                    self.depth += 1;
                    self.push(Tok::LParen, pos);
                    i += 1;
                }
                ')' => {
                    self.depth = self.depth.saturating_sub(1);
                    self.push(Tok::RParen, pos);
                    i += 1;
                }
                ',' => {
                    self.push(Tok::Comma, pos);
                    i += 1;
                }
                ':' if line.get(i + 1) == Some(&'=') => {
                    self.push(Tok::ColonEq, pos);
                    i += 2;
                }
                ':' => {
                    self.push(Tok::Colon, pos);
                    i += 1;
                }
                '-' if line.get(i + 1) == Some(&'>') => {
                    self.push(Tok::Arrow, pos);
                    i += 2;
                }
                '"' => i = self.string(line, i, line_no),
                '\'' => {
                    self.unsupported(pos, "a character literal", "lex.str.4");
                    i = line[i + 1..].iter().position(|c| *c == '\'').map_or(line.len(), |p| i + p + 2);
                }
                c if c.is_ascii_digit() => i = self.number(line, i, line_no),
                c if c.is_alphabetic() || c == '_' => {
                    let end = i + line[i..].iter().take_while(|c| c.is_alphanumeric() || **c == '_').count();
                    let word: String = line[i..end].iter().collect();
                    let tok = if word == "fn" { Tok::Fn } else { Tok::Ident(word) };
                    self.push(tok, pos);
                    i = end;
                }
                other => {
                    self.unsupported(pos, &format!("the `{other}` symbol"), "chapter 05");
                    i += 1;
                }
            }
        }
    }

    /// String literal starting at `line[start] == '"'`; returns the index after it.
    fn string(&mut self, line: &[char], start: usize, line_no: usize) -> usize {
        let pos = Pos { line: line_no, col: start + 1 };
        if line.get(start + 1) == Some(&'"') && line.get(start + 2) == Some(&'"') {
            self.unsupported(pos, "a multi-line string", "lex.str.3");
            return line.len();
        }
        let mut value = String::new();
        let mut i = start + 1;
        while i < line.len() {
            let here = Pos { line: line_no, col: i + 1 };
            match line[i] {
                '"' => {
                    self.push(Tok::Str(value), pos);
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
                    self.unsupported(here, "string interpolation", "lex.str.2");
                    value.push('{');
                    i += 1;
                }
                c => {
                    value.push(c);
                    i += 1;
                }
            }
        }
        self.diags.push(
            Diagnostic::new("lex.str.1", pos, "unterminated string literal").help("close the string with `\"`"),
        );
        line.len()
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
        if line.get(i + 1) == Some(&'u') && line.get(i + 2) == Some(&'{') {
            if let Some(close) = line[i + 3..].iter().position(|c| *c == '}') {
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
        let end = digits_start
            + line[digits_start..].iter().take_while(|c| c.is_ascii_alphanumeric() || **c == '_').count();
        let is_float = radix == 10
            && line.get(end) == Some(&'.')
            && line.get(end + 1).is_some_and(|c| c.is_ascii_digit());
        if is_float {
            self.unsupported(pos, "a floating-point literal", "lex.num.2");
            let frac = end + 1 + line[end + 1..].iter().take_while(|c| c.is_ascii_alphanumeric()).count();
            return frac;
        }
        let digits: String = line[digits_start..end].iter().filter(|c| **c != '_').collect();
        if radix == 10 && digits.contains(['e', 'E']) {
            self.unsupported(pos, "a floating-point literal", "lex.num.2");
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
            }
        }
        end
    }
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
                Fn,
                Ident("main".into()),
                LParen,
                RParen,
                Newline,
                Indent,
                Ident("print".into()),
                LParen,
                Str("Hello, world!".into()),
                RParen,
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
            vec![Fn, Ident("f".into()), LParen, RParen, Newline, Indent, Ident("g".into()), LParen, RParen, Newline, Dedent, Eof]
        );
    }

    #[test]
    fn newlines_inside_parens_and_after_commas_continue() {
        let t = toks("fn f()\n    g(1,\n        2)\n    h(3,\n      4)\n");
        let newlines = t.iter().filter(|t| **t == Tok::Newline).count();
        assert_eq!(newlines, 3); // header, g(...), h(...)
    }

    #[test]
    fn escapes_and_unicode() {
        assert_eq!(toks(r#""a\tb\n\"\\\{\u{1F600}""#)[0], Tok::Str("a\tb\n\"\\{\u{1F600}".into()));
    }

    #[test]
    fn integer_forms() {
        assert_eq!(toks("1_000 0xff 0b101 0o17"), vec![Tok::Int(1000), Tok::Int(255), Tok::Int(5), Tok::Int(15), Tok::Newline, Tok::Eof]);
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
        assert_eq!(rules("\"hi {name}\""), vec![UNSUPPORTED]);
    }
}
