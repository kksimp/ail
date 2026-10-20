//! Module syntax (ADRs 0097-0099): `use` declarations, the `pub` marker,
//! and qualified names `m.Name` in types and patterns (mod.use.1-3,
//! mod.vis.1-2).

use super::Parser;
use crate::ast::{Expr, Program, Use};
use crate::diag::Pos;
use crate::lexer::Tok;

impl Parser {
    /// `use a.b.c` or `use a.b.c as d`, on a line of its own (mod.use.1).
    /// Whether it comes before the file's declarations (mod.use.3) is
    /// checked with the package, which knows where each file starts.
    pub(super) fn use_decl(&mut self, prog: &mut Program) {
        let pos = self.bump().pos; // use
        let parsed = (|| {
            let mut path = vec![self.name("a module path after `use`", "mod.use.1")?];
            while self.eat_sym(".") {
                path.push(self.name("a module name after `.`", "mod.use.1")?);
            }
            let alias = if self.eat_kw("as") { Some(self.name("a name after `as`", "mod.use.1")?) } else { None };
            if !self.eat(&Tok::Newline) && !matches!(self.peek(), Tok::Eof) {
                self.unexpected("end of line after the import", "mod.use.1");
                return None;
            }
            Some(Use { path, alias, pos })
        })();
        match parsed {
            Some(u) => {
                self.aliases.insert(u.name().0.to_string());
                prog.uses.push(u);
            }
            None => self.skip_statement(),
        }
    }

    /// `pub` before a top-level declaration (mod.vis.1). The declaration
    /// follows on the same line, so the line marks it public.
    pub(super) fn pub_marker(&mut self) {
        let pos = self.bump().pos; // pub
        match (self.peek(), self.peek_at(1)) {
            (Tok::Kw("fn" | "struct" | "enum" | "error" | "trait"), _) | (Tok::Ident(_), Tok::Sym(":=")) => {
                self.public.insert(pos.line);
            }
            (Tok::Kw("impl"), _) => self.error_help(
                "mod.vis.2",
                pos,
                "an `impl` has no visibility of its own; its methods have the trait's visibility",
                "remove `pub`",
            ),
            (Tok::Kw("use"), _) => self.error_help(
                "mod.use.2",
                pos,
                "an import can't be re-exported; importers write their own `use`",
                "remove `pub`",
            ),
            _ => {
                self.unexpected(
                    "a declaration after `pub` (`fn`, `struct`, `enum`, `error`, `trait` or a constant)",
                    "mod.vis.1",
                );
                self.skip_statement();
            }
        }
    }

    /// `pub` before a member that takes its visibility from its trait or
    /// enum (mod.vis.2): reported and skipped.
    pub(super) fn member_pub(&mut self, what: &str) {
        if self.at_kw("pub") {
            let pos = self.bump().pos;
            self.error_help(
                "mod.vis.2",
                pos,
                format!("{what} have the visibility of their {}", owner_of(what)),
                "remove `pub`",
            );
        }
    }

    /// `pub` before a struct field (mod.vis.1).
    pub(super) fn field_pub(&mut self, rule: &'static str) {
        if !self.at_kw("pub") {
            return;
        }
        let pos = self.bump().pos;
        if rule == "type.struct.2" {
            self.public.insert(pos.line);
        } else {
            self.error_help("mod.vis.2", pos, "variant fields have the visibility of their enum", "remove `pub`");
        }
    }

    /// `Name`, or a qualified `m.Name` of an imported module (mod.use.2).
    pub(super) fn dotted_name(&mut self, expected: &str, rule: &'static str) -> Option<(String, Pos)> {
        let (mut name, pos) = self.name(expected, rule)?;
        while self.at_sym(".") && matches!(self.peek_at(1), Tok::Ident(_)) {
            self.i += 1;
            let (next, _) = self.name(expected, rule)?;
            name = format!("{name}.{next}");
        }
        Some((name, pos))
    }

    /// `m.Stack[int]` after `m.Stack`, where `m` is an import and `Stack` a
    /// generic name (gen.infer.2, mod.use.2): the type arguments.
    pub(super) fn qualified_type_args(&mut self, recv: &Expr, name: &str) -> Option<Option<Expr>> {
        let Expr::Name(m, pos) = recv else { return Some(None) };
        if !self.aliases.contains(m) || !self.at_sym("[") || !self.generic_names.contains(name) {
            return Some(None);
        }
        let (full, pos) = (format!("{m}.{name}"), *pos);
        self.bump(); // [
        let mut args = Vec::new();
        while !self.at_sym("]") {
            args.push(self.type_ref("a type argument", "gen.infer.2")?);
            if !self.eat_sym(",") {
                break;
            }
        }
        self.expect_sym("]", "`]` or `,` in the type arguments", "gen.infer.2")?;
        Some(Some(Expr::TypeArgs { name: full, args, pos }))
    }
}

fn owner_of(what: &str) -> &'static str {
    if what.contains("variant") { "enum" } else { "trait" }
}
