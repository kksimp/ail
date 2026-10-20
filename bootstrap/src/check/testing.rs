//! Attributes and test functions (test.attr.1-3, test.assert.1-2).

use super::{Checker, Ty};
use crate::ast::{Arg, BinOp, FnDecl, TypeKind};
use crate::diag::Pos;

/// The attributes the bootstrap knows (test.attr.1).
pub(super) const KNOWN_ATTRS: &[&str] = &["test", "pure", "inline", "deprecated"];

impl FnDecl {
    /// `@test fn name()` (test.attr.2).
    pub fn is_test(&self) -> bool {
        self.attrs.iter().any(|a| a.name == "test")
    }
}

impl<'p> Checker<'p> {
    /// test.attr.1-2: known attributes, and the shape of a test function.
    pub(super) fn check_attrs(&mut self) {
        for f in &self.prog.fns {
            for a in &f.attrs {
                if !KNOWN_ATTRS.contains(&a.name.as_str()) {
                    let known = KNOWN_ATTRS.iter().map(|k| format!("`@{k}`")).collect::<Vec<_>>().join(", ");
                    self.error_help(
                        "test.attr.1",
                        a.pos,
                        format!("unknown attribute `@{}`", a.name),
                        format!("the known attributes are {known}"),
                    );
                }
                // gram.attr.4: `@deprecated` carries the reason as a string.
                if a.name == "deprecated" && a.arg.is_none() {
                    self.error_help(
                        "gram.attr.4",
                        a.pos,
                        "`@deprecated` needs a reason",
                        "write `@deprecated(\"use new_name instead\")`",
                    );
                }
            }
            if !f.is_test() {
                continue;
            }
            let shape = if f.owner.is_some() || !f.top_level() {
                Some((f.pos, "a test is a plain top-level function, not a method"))
            } else if !f.generics.is_empty() {
                Some((f.generics[0].pos, "a test has no type parameters"))
            } else {
                None
            };
            if let Some((pos, msg)) = shape {
                self.error_help("test.attr.2", pos, msg, "write `@test fn name()`");
            }
            if shape.is_none() {
                self.check_prop_params(f);
            }
            if let Some(ret) = &f.ret
                && !matches!(&ret.kind, TypeKind::Fallible { ok: None, .. })
            {
                self.error_help(
                    "test.attr.2",
                    ret.pos,
                    "a test returns nothing, or only an error",
                    "write `@test fn name()`, or `@test fn name() !` if it can fail",
                );
            }
        }
    }

    /// test.prop.1-2: a test with parameters is a property, and each
    /// parameter's type must be one `ail test` can generate.
    fn check_prop_params(&mut self, f: &FnDecl) {
        for p in &f.params {
            let mut why = None;
            if p.mutable {
                why = Some("a property's parameters aren't `mut`".to_string());
            } else if let Some(bad) = self.ungeneratable(&p.ty, &mut Vec::new()) {
                why = Some(format!("`ail test` can't generate values of type `{bad}`"));
            }
            if let Some(msg) = why {
                self.error_help(
                    "test.prop.2",
                    p.pos,
                    msg,
                    "use `int`, `float`, `bool`, `char`, `str`, `List`, `Map`, `Set`, optionals, or structs and enums made of those",
                );
            }
        }
    }

    /// The first part of `t` that can't be generated (test.prop.2), as written.
    fn ungeneratable(&self, t: &crate::ast::TypeRef, seen: &mut Vec<String>) -> Option<String> {
        match &t.kind {
            TypeKind::Optional(inner) => self.ungeneratable(inner, seen),
            TypeKind::Named { name, args } => match name.as_str() {
                "int" | "i64" | "float" | "f64" | "str" | "char" | "bool" if args.is_empty() => None,
                "List" | "Set" | "Map" => args.iter().find_map(|a| self.ungeneratable(a, seen)),
                _ if !args.is_empty() => Some(name.clone()),
                _ if seen.contains(name) => None,
                _ => {
                    seen.push(name.clone());
                    if let Some(s) = self.prog.structs.iter().find(|s| s.name == *name) {
                        if s.is_error || !s.generics.is_empty() {
                            return Some(name.clone());
                        }
                        return s.fields.iter().find_map(|f| self.ungeneratable(&f.ty, seen));
                    }
                    if let Some(e) = self.prog.enums.iter().find(|e| e.name == *name) {
                        if e.is_error || !e.generics.is_empty() {
                            return Some(name.clone());
                        }
                        return e.variants.iter().flat_map(|v| &v.fields).find_map(|f| self.ungeneratable(&f.ty, seen));
                    }
                    Some(name.clone())
                }
            },
            TypeKind::Any(tr) => Some(format!("any {tr}")),
            TypeKind::Fallible { .. } => Some("a fallible type".to_string()),
        }
    }

    /// test.attr.3: a test is run by `ail test`, never called.
    pub(super) fn reject_test_call(&mut self, i: usize, pos: Pos) {
        let f = &self.prog.fns[i];
        if f.is_test() {
            let name = f.name.clone();
            self.error_help(
                "test.attr.3",
                pos,
                format!("`{name}` is a test and can't be called"),
                "move shared code into an ordinary function that the test calls",
            );
        }
    }

    /// `assert(cond)` and `assert_eq(a, b)` (test.assert.1-2).
    pub(super) fn assert_call(&mut self, name: &str, args: &'p [Arg], pos: Pos) -> Ty {
        let want = if name == "assert" { 1 } else { 2 };
        for a in args {
            if let Some((_, lpos)) = &a.label {
                self.unsupported(*lpos, "named arguments are not supported by the bootstrap yet (ADR 0058)");
            }
            if a.mutable {
                self.error_help(
                    "own.param.2",
                    a.value.pos(),
                    format!("`{name}`'s parameters aren't `mut`"),
                    "remove the `mut`",
                );
            }
        }
        if args.len() != want {
            self.args_only(args);
            let n = super::was_were(args.len());
            self.error("test.assert.2", pos, format!("`{name}` takes exactly {want} argument(s), but {n} given"));
            return Ty::Unit;
        }
        if name == "assert" {
            let t = self.expr(&args[0].value, Some(&Ty::Bool), true);
            if !matches!(t, Ty::Err | Ty::Never)
                && !self.reject_unhandled(&t, args[0].value.pos())
                && !self.accepts(&Ty::Bool, &t)
            {
                let msg = format!("`assert` needs a `bool`, found {}", self.show(&t));
                self.error("test.assert.2", args[0].value.pos(), msg);
            }
        } else {
            let lt = self.expr(&args[0].value, None, true);
            let rt = self.expr(&args[1].value, Some(&lt), true);
            // Same rules as `==`, and the values need a text form.
            self.operator(BinOp::Eq, &lt, &rt, pos);
            self.text_form(&lt, args[0].value.pos(), "test.assert.2");
        }
        Ty::Unit
    }
}

impl<'p> Checker<'p> {
    /// gram.attr.4: using a function marked `@deprecated("why")` warns, with the reason.
    pub(super) fn note_deprecated(&mut self, f: usize, pos: Pos) {
        let decl = &self.prog.fns[f];
        let Some(a) = decl.attrs.iter().find(|a| a.name == "deprecated") else { return };
        let Some(why) = a.arg.as_deref() else { return };
        let msg = format!("`{}` is deprecated: {why}", decl.name);
        self.diags.push(crate::diag::Diagnostic::warning("gram.attr.4", pos, msg));
    }
}
