//! A fast hasher for the interpreter's lookup tables (ADR 0175). The standard
//! hasher resists attacks on untrusted keys; these keys are names and source
//! positions from the program being run, and looking them up on every call
//! and literal made SipHash a visible share of the run time.

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

/// FNV-1a over the bytes written, with a word-at-a-time path for integers.
pub struct FastHasher(u64);

impl Default for FastHasher {
    fn default() -> Self {
        FastHasher(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for FastHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
        }
    }

    fn write_usize(&mut self, n: usize) {
        self.0 = (self.0 ^ n as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15).rotate_left(29);
    }

    // Integers (map keys, enum tags) in one step, not byte by byte.
    fn write_u8(&mut self, n: u8) {
        self.write_usize(usize::from(n));
    }

    fn write_u32(&mut self, n: u32) {
        self.write_usize(n as usize);
    }

    fn write_u64(&mut self, n: u64) {
        self.write_usize(n as usize);
    }

    fn write_i64(&mut self, n: i64) {
        self.write_usize(n as usize);
    }

    fn write_isize(&mut self, n: isize) {
        self.write_usize(n as usize);
    }
}

pub type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;
pub type FastSet<K> = HashSet<K, BuildHasherDefault<FastHasher>>;
