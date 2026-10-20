//! Private fields and methods of another module (mod.vis.1-2, ADR 0098).
//! Items reached by a qualified name are checked in `modules.rs`; members
//! need the receiver's type, so they are checked here.

use super::Checker;
use crate::ast::Within;
use crate::diag::Pos;

impl<'p> Checker<'p> {
    /// `x.f` or `x.f = ...` on a private field of a struct from another module.
    pub(super) fn private_field(&mut self, s: usize, field: usize, pos: Pos) {
        let decl = &self.prog.structs[s];
        let f = &decl.fields[field];
        if self.prog.hidden(f.pos, pos) {
            let msg = format!("field `{}` of `{}` is private to its module", f.name, decl.name);
            self.error_help("mod.vis.1", pos, msg, format!("mark the field `pub {} ...` to use it here", f.name));
        }
    }

    /// `T(...)` for a struct from another module with a private field: only
    /// its own module can build it.
    pub(super) fn private_literal(&mut self, s: usize, pos: Pos) {
        let decl = &self.prog.structs[s];
        if let Some(f) = decl.fields.iter().find(|f| self.prog.hidden(f.pos, pos)) {
            let msg = format!("`{}` can't be built here: its field `{}` is private to its module", decl.name, f.name);
            self.error_help("mod.vis.1", pos, msg, "call a `pub` function of its module that builds it");
        }
    }

    /// A call of a private method or associated function `fn T.f` of another
    /// module. Methods of `trait` and `impl` blocks have the trait's
    /// visibility (mod.vis.2), which naming the trait already checked.
    pub(super) fn private_fn(&mut self, i: usize, pos: Pos) {
        let f = &self.prog.fns[i];
        if f.within == Within::TopLevel && self.prog.hidden(f.pos, pos) {
            let full = f.full_name();
            // As written in its own module: `Account.audit`, not `lib.Account.audit`.
            let module = self.prog.module_at(f.pos);
            let local = full.strip_prefix(module).and_then(|n| n.strip_prefix('.')).unwrap_or(&full);
            let msg = format!("`{full}` is private to its module");
            self.error_help("mod.vis.1", pos, msg, format!("mark it `pub fn {local}` to use it here"));
        }
    }
}
