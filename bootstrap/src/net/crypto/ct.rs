//! Constant-time helpers (ADR 0366): no branch depends on the data.

/// Whether `a` and `b` are equal, comparing every byte whatever the result.
/// Different lengths are unequal; lengths are public.
pub fn eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut d = 0u8;
    for (x, y) in a.iter().zip(b) {
        d |= x ^ y;
    }
    // Keep the optimiser from turning the loop into an early exit.
    std::hint::black_box(d) == 0
}

/// All ones when `bit` is 1, zero when it is 0.
#[inline]
pub fn mask64(bit: u64) -> u64 {
    0u64.wrapping_sub(bit & 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_compares_contents_and_lengths() {
        assert!(eq(b"abc", b"abc"));
        assert!(!eq(b"abc", b"abd"));
        assert!(!eq(b"abc", b"ab"));
        assert!(eq(b"", b""));
        assert_eq!(mask64(1), u64::MAX);
        assert_eq!(mask64(0), 0);
    }
}
