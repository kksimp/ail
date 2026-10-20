//! `?` on an optional in a function whose errors are inferred: `none` fails
//! with the prelude's `error struct Absent` (err.prop.5, ADR 0279).

use super::{Checker, Errs, Ty};
use crate::diag::Pos;

impl<'p> Checker<'p> {
    /// err.prop.5: when the function returns `!T` or `!` with an inferred
    /// error set, `?` on an optional adds `Absent` to it and records `pos`.
    /// False when it doesn't apply, so the caller reports err.prop.2.
    pub(super) fn absent_try(&mut self, ret: &Ty, pos: Pos) -> bool {
        let Ty::Res(_, want @ Errs::Inferred(_)) = ret else { return false };
        let Some(super::Def::Struct(s)) = self.types.get("Absent").copied() else { return false };
        self.add_errors(want, &[Ty::Struct(s, Vec::new())]);
        self.resolved.absent_try.insert(pos);
        true
    }
}
