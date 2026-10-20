//! AES-128 and AES-256 encryption (FIPS 197), for AES-GCM (RFC 8446 §B.4).
//! Only the forward cipher: GCM never decrypts with AES.
//!
//! Constant time (ADR 0366): the S-box is the Boyar–Peralta depth-16
//! circuit evaluated on bit planes (bit `b` of byte `i` is bit `i` of plane
//! `b`), so no lookup table is indexed by secret bytes. MixColumns uses a
//! masked `xtime`.

/// Bit-plane S-box: `q[b]` holds bit `b` of each lane. The circuit is
/// from Boyar and Peralta, "A depth-16 circuit for the AES S-box" (2011).
fn sbox_planes(q: &mut [u16; 8]) {
    let (x0, x1, x2, x3, x4, x5, x6, x7) = (q[7], q[6], q[5], q[4], q[3], q[2], q[1], q[0]);

    // Top linear transformation.
    let y14 = x3 ^ x5;
    let y13 = x0 ^ x6;
    let y9 = x0 ^ x3;
    let y8 = x0 ^ x5;
    let t0 = x1 ^ x2;
    let y1 = t0 ^ x7;
    let y4 = y1 ^ x3;
    let y12 = y13 ^ y14;
    let y2 = y1 ^ x0;
    let y5 = y1 ^ x6;
    let y3 = y5 ^ y8;
    let t1 = x4 ^ y12;
    let y15 = t1 ^ x5;
    let y20 = t1 ^ x1;
    let y6 = y15 ^ x7;
    let y10 = y15 ^ t0;
    let y11 = y20 ^ y9;
    let y7 = x7 ^ y11;
    let y17 = y10 ^ y11;
    let y19 = y10 ^ y8;
    let y16 = t0 ^ y11;
    let y21 = y13 ^ y16;
    let y18 = x0 ^ y16;

    // Non-linear section.
    let t2 = y12 & y15;
    let t3 = y3 & y6;
    let t4 = t3 ^ t2;
    let t5 = y4 & x7;
    let t6 = t5 ^ t2;
    let t7 = y13 & y16;
    let t8 = y5 & y1;
    let t9 = t8 ^ t7;
    let t10 = y2 & y7;
    let t11 = t10 ^ t7;
    let t12 = y9 & y11;
    let t13 = y14 & y17;
    let t14 = t13 ^ t12;
    let t15 = y8 & y10;
    let t16 = t15 ^ t12;
    let t17 = t4 ^ t14;
    let t18 = t6 ^ t16;
    let t19 = t9 ^ t14;
    let t20 = t11 ^ t16;
    let t21 = t17 ^ y20;
    let t22 = t18 ^ y19;
    let t23 = t19 ^ y21;
    let t24 = t20 ^ y18;

    let t25 = t21 ^ t22;
    let t26 = t21 & t23;
    let t27 = t24 ^ t26;
    let t28 = t25 & t27;
    let t29 = t28 ^ t22;
    let t30 = t23 ^ t24;
    let t31 = t22 ^ t26;
    let t32 = t31 & t30;
    let t33 = t32 ^ t24;
    let t34 = t23 ^ t33;
    let t35 = t27 ^ t33;
    let t36 = t24 & t35;
    let t37 = t36 ^ t34;
    let t38 = t27 ^ t36;
    let t39 = t29 & t38;
    let t40 = t25 ^ t39;

    let t41 = t40 ^ t37;
    let t42 = t29 ^ t33;
    let t43 = t29 ^ t40;
    let t44 = t33 ^ t37;
    let t45 = t42 ^ t41;
    let z0 = t44 & y15;
    let z1 = t37 & y6;
    let z2 = t33 & x7;
    let z3 = t43 & y16;
    let z4 = t40 & y1;
    let z5 = t29 & y7;
    let z6 = t42 & y11;
    let z7 = t45 & y17;
    let z8 = t41 & y10;
    let z9 = t44 & y12;
    let z10 = t37 & y3;
    let z11 = t33 & y4;
    let z12 = t43 & y13;
    let z13 = t40 & y5;
    let z14 = t29 & y2;
    let z15 = t42 & y9;
    let z16 = t45 & y14;
    let z17 = t41 & y8;

    // Bottom linear transformation.
    let t46 = z15 ^ z16;
    let t47 = z10 ^ z11;
    let t48 = z5 ^ z13;
    let t49 = z9 ^ z10;
    let t50 = z2 ^ z12;
    let t51 = z2 ^ z5;
    let t52 = z7 ^ z8;
    let t53 = z0 ^ z3;
    let t54 = z6 ^ z7;
    let t55 = z16 ^ z17;
    let t56 = z12 ^ t48;
    let t57 = t50 ^ t53;
    let t58 = z4 ^ t46;
    let t59 = z3 ^ t54;
    let t60 = t46 ^ t57;
    let t61 = z14 ^ t57;
    let t62 = t52 ^ t58;
    let t63 = t49 ^ t58;
    let t64 = z4 ^ t59;
    let t65 = t61 ^ t62;
    let t66 = z1 ^ t63;
    let s0 = t59 ^ t63;
    let s6 = t56 ^ !t62;
    let s7 = t48 ^ !t60;
    let t67 = t64 ^ t65;
    let s3 = t53 ^ t66;
    let s4 = t51 ^ t66;
    let s5 = t47 ^ t65;
    let s1 = t64 ^ !s3;
    let s2 = t55 ^ !t67;

    *q = [s7, s6, s5, s4, s3, s2, s1, s0];
}

/// The S-box applied to up to 16 bytes at once.
fn sub_bytes(bytes: &mut [u8]) {
    debug_assert!(bytes.len() <= 16);
    let mut q = [0u16; 8];
    for (i, &x) in bytes.iter().enumerate() {
        for (b, plane) in q.iter_mut().enumerate() {
            *plane |= (((x >> b) & 1) as u16) << i;
        }
    }
    sbox_planes(&mut q);
    for (i, x) in bytes.iter_mut().enumerate() {
        let mut v = 0u8;
        for (b, plane) in q.iter().enumerate() {
            v |= (((plane >> i) & 1) as u8) << b;
        }
        *x = v;
    }
}

#[inline]
fn xtime(x: u8) -> u8 {
    (x << 1) ^ (0x1b & 0u8.wrapping_sub(x >> 7))
}

/// An expanded AES key (FIPS 197 §5.2).
#[derive(Clone)]
pub struct Aes {
    round_keys: Vec<[u8; 16]>,
}

impl Aes {
    /// A 16-byte (AES-128) or 32-byte (AES-256) key.
    pub fn new(key: &[u8]) -> Self {
        let nk = key.len() / 4;
        assert!(key.len() == 16 || key.len() == 32, "AES key must be 16 or 32 bytes");
        let rounds = nk + 6;
        let mut w: Vec<[u8; 4]> = key.chunks_exact(4).map(|c| c.try_into().unwrap()).collect();
        let mut rcon = 1u8;
        for i in nk..4 * (rounds + 1) {
            let mut t = w[i - 1];
            if i % nk == 0 {
                t.rotate_left(1);
                sub_bytes(&mut t);
                t[0] ^= rcon;
                rcon = xtime(rcon);
            } else if nk > 6 && i % nk == 4 {
                sub_bytes(&mut t);
            }
            let prev = w[i - nk];
            w.push(std::array::from_fn(|j| prev[j] ^ t[j]));
        }
        let round_keys = w.chunks_exact(4).map(|c| std::array::from_fn(|j| c[j / 4][j % 4])).collect();
        Aes { round_keys }
    }

    /// Encrypts one block in place (FIPS 197 §5.1).
    pub fn encrypt(&self, block: &mut [u8; 16]) {
        let last = self.round_keys.len() - 1;
        xor16(block, &self.round_keys[0]);
        for (round, rk) in self.round_keys.iter().enumerate().skip(1) {
            sub_bytes(block);
            // ShiftRows: byte r + 4c comes from column c + r.
            let s = *block;
            for c in 0..4 {
                for r in 1..4 {
                    block[r + 4 * c] = s[r + 4 * ((c + r) % 4)];
                }
            }
            if round != last {
                for col in block.chunks_exact_mut(4) {
                    let [a0, a1, a2, a3] = [col[0], col[1], col[2], col[3]];
                    let all = a0 ^ a1 ^ a2 ^ a3;
                    col[0] ^= all ^ xtime(a0 ^ a1);
                    col[1] ^= all ^ xtime(a1 ^ a2);
                    col[2] ^= all ^ xtime(a2 ^ a3);
                    col[3] ^= all ^ xtime(a3 ^ a0);
                }
            }
            xor16(block, rk);
        }
    }
}

#[inline]
fn xor16(a: &mut [u8; 16], b: &[u8; 16]) {
    for (x, y) in a.iter_mut().zip(b) {
        *x ^= y;
    }
}

#[cfg(test)]
mod tests {
    use super::super::hex;
    use super::*;

    /// The S-box from its definition (FIPS 197 §5.1.1): the inverse in
    /// GF(2^8), then the affine map. Slow and table-free; tests only.
    fn reference_sbox(x: u8) -> u8 {
        let mul = |mut a: u8, mut b: u8| {
            let mut p = 0u8;
            while b != 0 {
                if b & 1 != 0 {
                    p ^= a;
                }
                a = xtime(a);
                b >>= 1;
            }
            p
        };
        let mut inv = 1u8;
        if x == 0 {
            inv = 0;
        } else {
            for _ in 0..254 {
                inv = mul(inv, x);
            }
        }
        inv ^ inv.rotate_left(1) ^ inv.rotate_left(2) ^ inv.rotate_left(3) ^ inv.rotate_left(4) ^ 0x63
    }

    #[test]
    fn bitsliced_sbox_matches_definition() {
        for start in (0..256).step_by(16) {
            let mut bytes: Vec<u8> = (start..start + 16).map(|v| v as u8).collect();
            sub_bytes(&mut bytes);
            for (i, &s) in bytes.iter().enumerate() {
                let x = (start + i) as u8;
                assert_eq!(s, reference_sbox(x), "S-box({x:#04x})");
            }
        }
        assert_eq!(reference_sbox(0x53), 0xed); // FIPS 197 §5.1.1's example
    }

    // FIPS 197 Appendix C.1 and C.3.
    #[test]
    fn fips197_appendix_c() {
        let pt: [u8; 16] = hex("00112233445566778899aabbccddeeff").try_into().unwrap();
        let mut b = pt;
        Aes::new(&hex("000102030405060708090a0b0c0d0e0f")).encrypt(&mut b);
        assert_eq!(b.to_vec(), hex("69c4e0d86a7b0430d8cdb78070b4c55a"));
        let mut b = pt;
        Aes::new(&hex("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")).encrypt(&mut b);
        assert_eq!(b.to_vec(), hex("8ea2b7ca516745bfeafc49904b496089"));
    }

    // FIPS 197 Appendix A.1: the last round key of the 2b7e... key.
    #[test]
    fn fips197_key_expansion() {
        let aes = Aes::new(&hex("2b7e151628aed2a6abf7158809cf4f3c"));
        assert_eq!(aes.round_keys[10].to_vec(), hex("d014f9a8c9ee2589e13f0cc8b6630ca6"));
    }
}
