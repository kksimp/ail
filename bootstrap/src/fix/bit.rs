//! Fixes for the C-style bit operators (expr.bit.1, ADR 0465). AIL has no
//! `&`, `|` or `^` operators: bit operations are `int` methods. The rewrite
//! of `l op r` to `l.method(r)` only applies when the operands are whole
//! atoms of the line (`diag::rewrite`), so precedence can't change.

use crate::diag::{Edit, Pos};

/// The fix that rewrites `l OP r` at `pos` as `l.METHOD(r)`.
pub fn operator(pos: Pos, op: &str, method: &str) -> Edit {
    Edit::operands(pos, op, format!("$l.{method}($r)"))
}
