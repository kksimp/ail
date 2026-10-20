//! Fixes for braces in string literals (lex.str.2, spec chapter 02). Every
//! string interpolates `{expression}`, so a literal `{` is written `\{`. The
//! fix inserts the backslash before the brace; a `}` after it stays literal.

use crate::diag::{Edit, Pos};

/// The fix that writes the `{` at `pos` as `\{`.
pub fn literal_brace(pos: Pos) -> Edit {
    Edit::insert(pos, "\\")
}
