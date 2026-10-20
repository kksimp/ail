//! `requires` and `ensures` conditions (contract.decl.2, contract.ens.1-2,
//! ADR 0210-0211). Purity (contract.decl.3) is checked with the effect sets
//! in `effects::check_contracts`.

use super::{Checker, Kind, Local, Ty};
use crate::ast::FnDecl;

impl<'p> Checker<'p> {
    /// Check `f`'s contracts with its parameters in scope; `ensures` also
    /// sees `result`, the success value (contract.ens.1).
    pub(super) fn check_contracts(&mut self, f: &'p FnDecl, ret: &Ty) {
        for c in &f.contracts {
            let mark = self.locals.len();
            if c.ensures && !Self::returns_nothing(ret) {
                if let Some(p) = f.params.iter().find(|p| p.name == "result") {
                    self.error_help(
                        "contract.ens.2",
                        p.pos,
                        format!("`{}` has an `ensures` line, where `result` names its return value", f.full_name()),
                        "rename the parameter",
                    );
                } else {
                    let ty = Self::ok_type(ret);
                    self.locals.push(Local {
                        name: "result".into(),
                        ty,
                        mutable: false,
                        kind: Kind::Binding,
                        decl: None,
                    });
                }
            }
            let got = self.expr(&c.cond, Some(&Ty::Bool), true);
            if !self.accepts(&Ty::Bool, &got) {
                self.mismatch("contract.decl.2", c.cond.pos(), &Ty::Bool, &got);
            }
            self.locals.truncate(mark);
        }
    }
}
