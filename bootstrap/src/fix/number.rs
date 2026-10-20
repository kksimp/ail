//! Fixes for number literals (lex.num, expr.bind). A whole-number float
//! literal such as `2.0` where an `int` is expected is written `2`. The
//! anchored edit matches the literal as written with one decimal place, so a
//! literal spelled another way is left alone.

use crate::diag::{Edit, Fix, Pos};

/// The fix that writes the whole-number float `value` at `pos` as an `int`.
pub fn whole_float(pos: Pos, value: f64) -> Fix {
    Fix {
        title: format!("write `{}` as an int", value as i64),
        edits: vec![Edit::replace(pos, format!("{value:.1}"), (value as i64).to_string())],
    }
}
