//! Money is whole cents; reports print it as dollars (spec "Prices").

/// `p` percent of `a`, in whole cents. `a` is never negative.
pub fn percent(a: i64, p: i64) -> i64 {
    a * p / 100
}

/// Cents as dollars with exactly two decimals: `1234.05`, `0.50`, `-0.50`.
pub fn show(c: i64) -> String {
    let sign = if c < 0 { "-" } else { "" };
    let a = c.abs();
    format!("{}{}.{:02}", sign, a / 100, a % 100)
}
