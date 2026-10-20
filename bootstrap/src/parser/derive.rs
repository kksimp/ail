//! `@derive(generator) trait Name` (ct.derive.1, ADR 0227).

use super::Parser;
use crate::ast::Program;
use crate::lexer::Tok;

impl Parser {
    /// At `@`: `@derive(f) trait Name ...` declares a trait whose empty
    /// `impl`s call the generator `f` (ct.derive.1). Returns false, consuming
    /// nothing, when the line declares no trait, so `@derive` before a
    /// function stays an unknown attribute (test.attr.1).
    pub(super) fn derive_trait(&mut self, prog: &mut Program) -> bool {
        if !matches!(self.peek_at(1), Tok::Ident(n) if n == "derive") {
            return false;
        }
        let shape = matches!(
            (self.peek_at(2), self.peek_at(3), self.peek_at(4), self.peek_at(5)),
            (Tok::Sym("("), Tok::Ident(_), Tok::Sym(")"), Tok::Kw("trait"))
        );
        if !shape {
            let line_has_trait = self.tokens[self.i..]
                .iter()
                .take_while(|t| !matches!(t.tok, Tok::Newline | Tok::Eof))
                .any(|t| matches!(t.tok, Tok::Kw("trait")));
            if !line_has_trait {
                return false;
            }
            let pos = self.pos();
            self.error_help(
                "ct.derive.1",
                pos,
                "`@derive` before a trait names one generator function",
                "write `@derive(generator) trait Name`",
            );
            self.skip_statement();
            return true;
        }
        let Tok::Ident(name) = self.peek_at(3).clone() else { unreachable!("matched above") };
        let pos = self.tokens[self.i + 3].pos;
        self.i += 5;
        let before = prog.traits.len();
        self.trait_decl(prog);
        if let Some(t) = prog.traits.get_mut(before) {
            t.derive = Some((name, pos));
        }
        true
    }
}
