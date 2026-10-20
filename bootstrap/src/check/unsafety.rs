//! `unsafe` blocks and functions (unsafe.block.1-3, ffi.call.1, ADRs
//! 0102, 0104, 0430): an unsafe operation is allowed only inside an `unsafe`
//! block, and only a package whose manifest allows it may contain one.

use super::Checker;
use crate::diag::Pos;

impl<'p> Checker<'p> {
    /// unsafe.block.3: this position's package may contain `unsafe` code.
    pub(super) fn unsafe_opt_in(&mut self, pos: Pos) {
        if self.prog.unsafe_ok_at(pos) {
            return;
        }
        let who = match self.prog.package_at(pos) {
            "" => "this package".to_string(),
            p => format!("dependency `{p}`"),
        };
        self.error_help(
            "unsafe.block.3",
            pos,
            format!("{who} contains `unsafe` code, which its manifest doesn't allow"),
            "add `unsafe allow` to the `package` line (or the `dep` line) of `ail.pkg`, or remove the `unsafe` code",
        );
    }

    /// unsafe.block.2, ffi.call.1: calling an `unsafe fn` or a C function is
    /// an unsafe operation (unsafe.op.1).
    pub(super) fn note_unsafe_call(&mut self, f: usize, pos: Pos) {
        let decl = &self.prog.fns[f];
        if self.unsafe_depth > 0 || !(decl.unsafe_fn || decl.ext.is_some()) {
            return;
        }
        let (rule, what) = if decl.ext.is_some() {
            ("ffi.call.1", format!("`{}` is a C function, and calling one", decl.name))
        } else {
            ("unsafe.block.2", format!("`{}` is an `unsafe fn`, and calling one", decl.name))
        };
        self.error_help(
            rule,
            pos,
            format!("{what} is an unsafe operation, allowed only inside an `unsafe` block"),
            "wrap the call in `unsafe`, ideally inside a safe AIL function that checks the preconditions",
        );
    }
}
