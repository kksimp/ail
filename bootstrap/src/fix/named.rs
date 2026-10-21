//! Fixes for named call arguments (expr.call.3, ADR 0454). A positional
//! argument after a named one fills the parameter at its own position, so
//! naming it with that parameter's name says what the call already means. The
//! fix is offered only when no named argument already uses that name.

use crate::diag::{Edit, Pos};

/// The fix that names the positional argument at `pos` as `name`.
pub fn name_positional(pos: Pos, name: &str) -> Edit {
    Edit::insert(pos, format!("{name}: "))
}
