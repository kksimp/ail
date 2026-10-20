//! X25519 (RFC 7748 §5), the key share for the TLS 1.3 handshake
//! (RFC 8446 §4.2.8.2). Field elements are five 51-bit limbs; the ladder
//! swaps with masks and runs the same steps for every scalar (ADR 0366).

type Fe = [u64; 5];

const M51: u64 = (1 << 51) - 1;

fn carry(mut h: Fe) -> Fe {
    for i in 0..4 {
        h[i + 1] += h[i] >> 51;
        h[i] &= M51;
    }
    h[0] += (h[4] >> 51) * 19;
    h[4] &= M51;
    h[1] += h[0] >> 51;
    h[0] &= M51;
    h
}

fn add(a: &Fe, b: &Fe) -> Fe {
    carry(std::array::from_fn(|i| a[i] + b[i]))
}

/// a - b, computed as a + 4p - b so no limb underflows.
fn sub(a: &Fe, b: &Fe) -> Fe {
    const FOUR_P: Fe = [0x1fffffffffffb4, 0x1ffffffffffffc, 0x1ffffffffffffc, 0x1ffffffffffffc, 0x1ffffffffffffc];
    carry(std::array::from_fn(|i| a[i] + FOUR_P[i] - b[i]))
}

fn mul(a: &Fe, b: &Fe) -> Fe {
    let m = |x: u64, y: u64| x as u128 * y as u128;
    let b19: [u64; 5] = std::array::from_fn(|i| b[i] * 19);
    let t0 = m(a[0], b[0]) + m(a[1], b19[4]) + m(a[2], b19[3]) + m(a[3], b19[2]) + m(a[4], b19[1]);
    let t1 = m(a[0], b[1]) + m(a[1], b[0]) + m(a[2], b19[4]) + m(a[3], b19[3]) + m(a[4], b19[2]);
    let t2 = m(a[0], b[2]) + m(a[1], b[1]) + m(a[2], b[0]) + m(a[3], b19[4]) + m(a[4], b19[3]);
    let t3 = m(a[0], b[3]) + m(a[1], b[2]) + m(a[2], b[1]) + m(a[3], b[0]) + m(a[4], b19[4]);
    let t4 = m(a[0], b[4]) + m(a[1], b[3]) + m(a[2], b[2]) + m(a[3], b[1]) + m(a[4], b[0]);
    let mut t = [t0, t1, t2, t3, t4];
    for i in 0..4 {
        t[i + 1] += t[i] >> 51;
        t[i] &= M51 as u128;
    }
    let top = (t[4] >> 51) as u64;
    t[4] &= M51 as u128;
    let mut h: Fe = std::array::from_fn(|i| t[i] as u64);
    h[0] += top * 19;
    carry(h)
}

fn square(a: &Fe) -> Fe {
    mul(a, a)
}

/// a^(p-2) = 1/a (Fermat). The exponent is public, so its bits may steer
/// the loop.
fn invert(a: &Fe) -> Fe {
    // p - 2 = 2^255 - 21: bits 254..5 are 1, then 01011.
    let mut r: Fe = [1, 0, 0, 0, 0];
    for bit in (0..255).rev() {
        r = square(&r);
        let set = bit >= 5 || (0b01011 >> bit) & 1 == 1;
        if set {
            r = mul(&r, a);
        }
    }
    r
}

fn cswap(swap: u64, a: &mut Fe, b: &mut Fe) {
    let mask = 0u64.wrapping_sub(swap);
    for i in 0..5 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
    }
}

fn from_bytes(s: &[u8; 32]) -> Fe {
    let load = |i: usize| u64::from_le_bytes(s[i..i + 8].try_into().unwrap());
    [
        load(0) & M51,
        (load(6) >> 3) & M51,
        (load(12) >> 6) & M51,
        (load(19) >> 1) & M51,
        (load(24) >> 12) & M51, // the top bit is ignored (RFC 7748 §5)
    ]
}

fn to_bytes(h: &Fe) -> [u8; 32] {
    let mut h = carry(carry(*h));
    // Subtract p if h >= p: q is 1 exactly then.
    let mut q = (h[0] + 19) >> 51;
    for limb in &h[1..] {
        q = (limb + q) >> 51;
    }
    h[0] += 19 * q;
    for i in 0..4 {
        h[i + 1] += h[i] >> 51;
        h[i] &= M51;
    }
    h[4] &= M51;
    let lo = h[0] | (h[1] << 51);
    let m1 = (h[1] >> 13) | (h[2] << 38);
    let m2 = (h[2] >> 26) | (h[3] << 25);
    let hi = (h[3] >> 39) | (h[4] << 12);
    let mut out = [0u8; 32];
    for (i, w) in [lo, m1, m2, hi].iter().enumerate() {
        out[8 * i..8 * i + 8].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// X25519(k, u) (RFC 7748 §5): the scalar is clamped here.
pub fn x25519(scalar: &[u8; 32], u: &[u8; 32]) -> [u8; 32] {
    let mut k = *scalar;
    k[0] &= 248;
    k[31] &= 127;
    k[31] |= 64;
    let x1 = from_bytes(u);
    let (mut x2, mut z2, mut x3, mut z3): (Fe, Fe, Fe, Fe) = ([1, 0, 0, 0, 0], [0; 5], x1, [1, 0, 0, 0, 0]);
    let a24: Fe = [121665, 0, 0, 0, 0];
    let mut swap = 0u64;
    for t in (0..255).rev() {
        let bit = ((k[t / 8] >> (t % 8)) & 1) as u64;
        swap ^= bit;
        cswap(swap, &mut x2, &mut x3);
        cswap(swap, &mut z2, &mut z3);
        swap = bit;
        let a = add(&x2, &z2);
        let aa = square(&a);
        let b = sub(&x2, &z2);
        let bb = square(&b);
        let e = sub(&aa, &bb);
        let c = add(&x3, &z3);
        let d = sub(&x3, &z3);
        let da = mul(&d, &a);
        let cb = mul(&c, &b);
        x3 = square(&add(&da, &cb));
        z3 = mul(&x1, &square(&sub(&da, &cb)));
        x2 = mul(&aa, &bb);
        z2 = mul(&e, &add(&aa, &mul(&a24, &e)));
    }
    cswap(swap, &mut x2, &mut x3);
    cswap(swap, &mut z2, &mut z3);
    to_bytes(&mul(&x2, &invert(&z2)))
}

/// The public key for a private scalar: X25519(k, 9).
pub fn public_key(scalar: &[u8; 32]) -> [u8; 32] {
    let mut nine = [0u8; 32];
    nine[0] = 9;
    x25519(scalar, &nine)
}

#[cfg(test)]
mod tests {
    use super::super::hex;
    use super::*;

    fn b32(s: &str) -> [u8; 32] {
        hex(s).try_into().unwrap()
    }

    // RFC 7748 §5.2, the two single-call vectors.
    #[test]
    fn rfc7748_vectors() {
        assert_eq!(
            x25519(
                &b32("a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4"),
                &b32("e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c")
            ),
            b32("c3da55379de9c6908e94ea4df28d084f32eccf03491c71f754b4075577a28552")
        );
        assert_eq!(
            x25519(
                &b32("4b66e9d4d1b4673c5ad22691957d6af5c11b6421e0ea01d42ca4169e7918ba0d"),
                &b32("e5210f12786811d3f4b7959d0538ae2c31dbe7106fc03c3efc4cd549c715a493")
            ),
            b32("95cbde9476e8907d7aade45cb4b873f88b595a68799fa152e6f8f7647aac7957")
        );
    }

    // RFC 7748 §5.2, the iterated vector (1 and 1,000 rounds).
    #[test]
    fn rfc7748_iterated() {
        let mut k = [0u8; 32];
        k[0] = 9;
        let mut u = k;
        for i in 1..=1000 {
            let r = x25519(&k, &u);
            u = k;
            k = r;
            if i == 1 {
                assert_eq!(k, b32("422c8e7a6227d7bca1350b3e2bb7279f7897b87bb6854b783c60e80311ae3079"));
            }
        }
        assert_eq!(k, b32("684cf59ba83309552800ef566f2f4d3c1c3887c49360e3875f2eb94d99532c51"));
    }

    // RFC 7748 §6.1: Alice and Bob agree.
    #[test]
    fn rfc7748_diffie_hellman() {
        let a = b32("77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a");
        let b = b32("5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb");
        let pa = public_key(&a);
        let pb = public_key(&b);
        assert_eq!(pa, b32("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a"));
        assert_eq!(pb, b32("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f"));
        let shared = b32("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742");
        assert_eq!(x25519(&a, &pb), shared);
        assert_eq!(x25519(&b, &pa), shared);
    }
}
