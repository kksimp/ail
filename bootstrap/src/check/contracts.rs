//! `requires` and `ensures` conditions (contract.decl.2, contract.ens.1-2,
//! ADR 0210-0211). Purity (contract.decl.3) is checked with the effect sets
//! in `effects::check_contracts`.

use super::{Checker, Kind, Local, Ty};
use crate::ast::FnDecl;

impl<'p> Checker<'p> {
    /// A trait method without a body still has its contracts checked
    /// (contract.inh.1), with its parameters in scope.
    pub(super) fn check_bodyless_contracts(&mut self, idx: usize, f: &'p FnDecl) {
        let sig = self.sigs[idx].clone();
        self.scope = self.fn_scopes[idx].clone();
        self.self_ty = self.fn_self[idx].clone();
        self.locals.clear();
        if f.receiver.is_some() {
            let ty = self.self_ty.clone().unwrap_or(Ty::Err);
            let mutable = f.receiver.is_some_and(|r| r.mutable);
            self.locals.push(Local { name: "self".into(), ty, mutable, kind: Kind::SelfParam, decl: None });
        }
        for (p, (_, ty)) in f.params.iter().zip(&sig.params) {
            self.locals.push(Local { name: p.name.clone(), ty: ty.clone(), mutable: p.mutable, kind: Kind::Param, decl: None });
        }
        self.check_contracts(f, &sig.ret);
    }

    /// contract.inv.1: a struct's invariants see its fields by bare name
    /// (rewritten to `self.name`) and must be `bool`.
    pub(super) fn check_invariants(&mut self, struct_index: usize) {
        let st = &self.prog.structs[struct_index];
        if st.invariants.is_empty() {
            return;
        }
        self.locals.clear();
        let declared = self.struct_params[struct_index].clone();
        self.scope = self.scope_of(&declared);
        let ty = Ty::Struct(struct_index, declared.iter().map(|&p| Ty::Param(p)).collect());
        self.self_ty = Some(ty.clone());
        self.locals.push(Local { name: "self".into(), ty, mutable: false, kind: Kind::SelfParam, decl: None });
        for c in &st.invariants {
            let got = self.expr(&c.cond, Some(&Ty::Bool), true);
            if !self.accepts(&Ty::Bool, &got) {
                self.mismatch("contract.decl.2", c.cond.pos(), &Ty::Bool, &got);
            }
        }
        self.locals.clear();
    }

    /// Check `f`'s contracts with its parameters in scope; `ensures` also
    /// sees `result`, the success value (contract.ens.1).
    pub(super) fn check_contracts(&mut self, f: &'p FnDecl, ret: &Ty) {
        for c in &f.contracts {
            let mark = self.locals.len();
            // contract.ens.3: `old(e)` is `e` typed in the parameters' scope.
            let mut olds = Vec::new();
            for (name, e) in &c.olds {
                let ty = self.expr(e, None, true);
                olds.push((name.clone(), ty));
            }
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
            for (name, ty) in olds {
                self.locals.push(Local { name, ty, mutable: false, kind: Kind::Binding, decl: None });
            }
            self.in_requires = !c.ensures;
            self.requires_pos = Some(c.pos);
            let got = self.expr(&c.cond, Some(&Ty::Bool), true);
            self.in_requires = false;
            if !self.accepts(&Ty::Bool, &got) {
                self.mismatch("contract.decl.2", c.cond.pos(), &Ty::Bool, &got);
            }
            self.locals.truncate(mark);
        }
    }
}
