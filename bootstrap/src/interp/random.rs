//! The generator behind `std.random` (std.random.1-5, ADR 0206): SplitMix64
//! over one process-wide state, so a seed fixes every later draw
//! (std.random.3). Unseeded, the state starts from the clock.

use std::sync::atomic::{AtomicU64, Ordering};

const GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// The generator state; 0 means "not seeded yet" (seeded from the clock on
/// first use).
static STATE: AtomicU64 = AtomicU64::new(0);

/// `random.seed(s)` and `ail run --seed S` (std.random.3, eff.det.3).
pub fn seed(s: i64) {
    // Mix the seed so nearby seeds give unrelated streams, and keep it nonzero.
    let mut z = (s as u64) ^ 0xD1B5_4A32_D192_ED03;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    STATE.store(z | 1, Ordering::Relaxed);
}

fn next() -> u64 {
    if STATE.load(Ordering::Relaxed) == 0 {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
        let _ = STATE.compare_exchange(0, t | 1, Ordering::Relaxed, Ordering::Relaxed);
    }
    let mut z = STATE.fetch_add(GAMMA, Ordering::Relaxed).wrapping_add(GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A float in `[0, 1)` (std.random.2).
pub fn unit() -> f64 {
    (next() >> 11) as f64 / (1u64 << 53) as f64
}

/// A uniform number in `0..n` for `n >= 1`, without modulo bias.
pub fn below(n: u64) -> u64 {
    let zone = u64::MAX - (u64::MAX % n + 1) % n;
    loop {
        let x = next();
        if x <= zone {
            return x % n;
        }
    }
}

/// A uniform int in `lo..=hi` (std.random.2); the caller checked `lo <= hi`.
pub fn range(lo: i64, hi: i64) -> i64 {
    let span = (hi as i128 - lo as i128 + 1) as u128;
    if span > u64::MAX as u128 {
        return next() as i64;
    }
    (lo as i128 + below(span as u64) as i128) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_streams_repeat() {
        seed(7);
        let a: Vec<i64> = (0..5).map(|_| range(1, 6)).collect();
        seed(7);
        let b: Vec<i64> = (0..5).map(|_| range(1, 6)).collect();
        assert_eq!(a, b);
        assert!(a.iter().all(|x| (1..=6).contains(x)));
        assert!((0..100).all(|_| (0.0..1.0).contains(&unit())));
        assert_eq!(range(i64::MIN, i64::MAX) as i128 >= i64::MIN as i128, true);
    }
}
