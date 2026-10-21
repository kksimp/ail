//! Messages written on error types (err.msg.1-2, ADR 0278):
//! `error struct E "text {field}"` and `Variant(f T) "text {f}"` in an
//! `error enum`. Each is the type's display text: the parser turns the
//! messages into the tokens of an `impl ToStr for E`, spliced in after the
//! declaration, so the checker and the interpreter see an ordinary impl.

use super::Parser;
use crate::diag::Pos;
use crate::lexer::{Part, Tok, Token};

impl Parser {
    /// The message after an error type's name or one of its variants, if the
    /// next token is a string (err.msg.1).
    pub(super) fn error_message(&mut self, is_error: bool) -> Option<Token> {
        if !matches!(self.peek(), Tok::Str(_) | Tok::Interp(_)) {
            return None;
        }
        let message = self.bump();
        if !is_error {
            self.error_help(
                "err.msg.1",
                message.pos,
                "only `error` types have messages",
                "declare it with `error struct` or `error enum`, or write `impl ToStr`",
            );
            return None;
        }
        Some(message)
    }

    /// err.msg.2: every variant has a message, or none does. Splices the
    /// `impl ToStr` that the messages define after the declaration.
    pub(super) fn splice_messages(
        &mut self,
        name: &str,
        pos: Pos,
        generic: bool,
        arms: Vec<(String, Vec<String>, Option<Token>)>,
        whole: Option<Token>,
    ) {
        let with = arms.iter().filter(|(_, _, m)| m.is_some()).count();
        if whole.is_none() && with == 0 {
            return;
        }
        if generic {
            self.error_help(
                "err.msg.2",
                pos,
                format!("`{name}` has type parameters, so it can't have messages"),
                "write `impl ToStr` for it instead",
            );
            return;
        }
        if with != 0 && with != arms.len() {
            let (missing, _, _) = arms.iter().find(|(_, _, m)| m.is_none()).expect("a variant without a message");
            self.error_help(
                "err.msg.2",
                pos,
                format!("some variants of `{name}` have a message but `{missing}` doesn't"),
                "give every variant a message, or none (and write `impl ToStr`)",
            );
            return;
        }
        let at = |tok: Tok| Token { tok, pos };
        let ident = |s: &str| at(Tok::Ident(s.to_string()));
        let mut t = vec![
            at(Tok::Kw("impl")),
            ident("ToStr"),
            at(Tok::Kw("for")),
            ident(name),
            at(Tok::Newline),
            at(Tok::Indent),
            at(Tok::Kw("fn")),
            ident("to_str"),
            at(Tok::Sym("(")),
            at(Tok::Kw("self")),
            at(Tok::Sym(")")),
            ident("str"),
            at(Tok::Newline),
            at(Tok::Indent),
        ];
        if let Some(m) = whole {
            // A struct's fields are in scope by bare name (ADR 0141).
            t.push(m);
            t.push(at(Tok::Newline));
        } else {
            t.extend([at(Tok::Kw("match")), at(Tok::Kw("self")), at(Tok::Newline), at(Tok::Indent)]);
            for (variant, fields, message) in arms {
                let message = message.expect("every variant has a message");
                let used = names_in(&message.tok);
                t.push(ident(&variant));
                if !fields.is_empty() {
                    t.push(at(Tok::Sym("(")));
                    for (i, f) in fields.iter().enumerate() {
                        if i > 0 {
                            t.push(at(Tok::Sym(",")));
                        }
                        t.push(ident(if used.contains(f) { f } else { "_" }));
                    }
                    t.push(at(Tok::Sym(")")));
                }
                t.push(at(Tok::Sym("=>")));
                t.push(message);
                t.push(at(Tok::Newline));
            }
            t.push(at(Tok::Dedent));
        }
        t.extend([at(Tok::Dedent), at(Tok::Dedent)]);
        self.tokens.splice(self.i..self.i, t);
    }
}

/// The plain names used inside a string's `{...}` parts.
fn names_in(tok: &Tok) -> Vec<String> {
    let Tok::Interp(parts) = tok else { return Vec::new() };
    let mut names = Vec::new();
    for part in parts {
        let (Part::Code(ts) | Part::Fmt(ts, _)) = part else { continue };
        for t in ts {
            match &t.tok {
                Tok::Ident(n) => names.push(n.clone()),
                Tok::Interp(_) => names.extend(names_in(&t.tok)),
                _ => {}
            }
        }
    }
    names
}
