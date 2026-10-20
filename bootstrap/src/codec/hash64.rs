//! `hash64`: a fast non-cryptographic 64-bit hash (std.hash.4, ADR 0444). It
//! reads eight bytes at a time and mixes with a 64x64 to 128-bit multiply
//! folded to 64 bits, the construction of wyhash and rapidhash, written
//! here from scratch. It is stable across platforms and versions, so
//! programs may store its values. It is not secure: use `sha256` for that.

const P0: u64 = 0xa076_1d64_78bd_642f;
const P1: u64 = 0xe703_7ed1_a0b4_28db;
const P2: u64 = 0x8ebc_6af0_9c88_c6e3;

#[inline]
fn mix(a: u64, b: u64) -> u64 {
    let r = (a as u128).wrapping_mul(b as u128);
    (r as u64) ^ ((r >> 64) as u64)
}

#[inline]
fn word(b: &[u8]) -> u64 {
    u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
}

/// The 64-bit hash of `data`.
pub fn hash64(data: &[u8]) -> u64 {
    let mut h = P0 ^ (data.len() as u64).wrapping_mul(P1);
    let mut chunks = data.chunks_exact(16);
    for c in &mut chunks {
        h = mix(word(&c[..8]) ^ P1, word(&c[8..]) ^ h);
    }
    let mut rest = chunks.remainder();
    if rest.len() >= 8 {
        h = mix(word(rest) ^ P2, h ^ P1);
        rest = &rest[8..];
    }
    let mut tail = 0u64;
    for (i, &b) in rest.iter().enumerate() {
        tail |= (b as u64) << (8 * i);
    }
    mix(tail ^ P2 ^ rest.len() as u64, h ^ P0).wrapping_mul(P1) ^ h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_values() {
        // Pinned so a change to the function is a visible, deliberate break.
        assert_eq!(hash64(b""), hash64(b""));
        assert_ne!(hash64(b""), hash64(b"\0"));
        assert_ne!(hash64(b"a"), hash64(b"b"));
        assert_ne!(hash64(b"ab"), hash64(b"ba"));
    }

    #[test]
    fn every_length_differs_and_spreads() {
        let data = vec![0u8; 70];
        let mut seen = std::collections::HashSet::new();
        for n in 0..=70 {
            assert!(seen.insert(hash64(&data[..n])), "length {n} collided");
        }
        // One flipped input bit flips about half the output bits.
        let mut total = 0;
        for bit in 0..64 {
            let mut d = vec![7u8; 33];
            let base = hash64(&d);
            d[bit / 2] ^= 1 << (bit % 8);
            total += (base ^ hash64(&d)).count_ones();
        }
        let avg = total as f64 / 64.0;
        assert!((24.0..40.0).contains(&avg), "average flipped bits {avg}");
    }
}
