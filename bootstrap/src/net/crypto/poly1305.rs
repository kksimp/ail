//! Poly1305 (RFC 8439 §2.5): a one-time authenticator. Three 44/44/42-bit
//! limbs multiplied in `u128`, no tables and no secret-dependent branches
//! (ADR 0366).

const M44: u64 = (1 << 44) - 1;
const M42: u64 = (1 << 42) - 1;

/// A Poly1305 computation under a 32-byte one-time key.
pub struct Poly1305 {
    r: [u64; 3],
    s: [u64; 2],
    h: [u64; 3],
    pad: [u64; 2],
    buf: [u8; 16],
    used: usize,
}

impl Poly1305 {
    pub fn new(key: &[u8; 32]) -> Self {
        let t0 = u64::from_le_bytes(key[0..8].try_into().unwrap());
        let t1 = u64::from_le_bytes(key[8..16].try_into().unwrap());
        // Clamping (RFC 8439 §2.5.1), folded into the limb split.
        let r0 = t0 & 0xffc0fffffff;
        let r1 = ((t0 >> 44) | (t1 << 20)) & 0xfffffc0ffff;
        let r2 = (t1 >> 24) & 0x00ffffffc0f;
        Poly1305 {
            r: [r0, r1, r2],
            s: [r1 * (5 << 2), r2 * (5 << 2)],
            h: [0; 3],
            pad: [
                u64::from_le_bytes(key[16..24].try_into().unwrap()),
                u64::from_le_bytes(key[24..32].try_into().unwrap()),
            ],
            buf: [0; 16],
            used: 0,
        }
    }

    /// Adds one 16-byte block; `hibit` is 2^128's bit (0 for the padded
    /// last block, which carries its own 0x01).
    fn block(&mut self, m: &[u8], hibit: u64) {
        let t0 = u64::from_le_bytes(m[0..8].try_into().unwrap());
        let t1 = u64::from_le_bytes(m[8..16].try_into().unwrap());
        let [r0, r1, r2] = self.r;
        let [s1, s2] = self.s;
        let mut h0 = self.h[0] + (t0 & M44);
        let mut h1 = self.h[1] + (((t0 >> 44) | (t1 << 20)) & M44);
        let mut h2 = self.h[2] + (((t1 >> 24) & M42) | hibit);
        let m = |a: u64, b: u64| a as u128 * b as u128;
        let d0 = m(h0, r0) + m(h1, s2) + m(h2, s1);
        let mut d1 = m(h0, r1) + m(h1, r0) + m(h2, s2);
        let mut d2 = m(h0, r2) + m(h1, r1) + m(h2, r0);
        h0 = d0 as u64 & M44;
        d1 += d0 >> 44;
        h1 = d1 as u64 & M44;
        d2 += d1 >> 44;
        h2 = d2 as u64 & M42;
        h0 += (d2 >> 42) as u64 * 5;
        h1 += h0 >> 44;
        h0 &= M44;
        self.h = [h0, h1, h2];
    }

    pub fn update(&mut self, mut data: &[u8]) {
        if self.used > 0 {
            let n = (16 - self.used).min(data.len());
            self.buf[self.used..self.used + n].copy_from_slice(&data[..n]);
            self.used += n;
            data = &data[n..];
            if self.used < 16 {
                return;
            }
            let b = self.buf;
            self.block(&b, 1 << 40);
            self.used = 0;
        }
        let mut blocks = data.chunks_exact(16);
        for b in &mut blocks {
            self.block(b, 1 << 40);
        }
        let rest = blocks.remainder();
        self.buf[..rest.len()].copy_from_slice(rest);
        self.used = rest.len();
    }

    pub fn finish(mut self) -> [u8; 16] {
        if self.used > 0 {
            let mut b = [0u8; 16];
            b[..self.used].copy_from_slice(&self.buf[..self.used]);
            b[self.used] = 1;
            self.block(&b, 0);
        }
        let [mut h0, mut h1, mut h2] = self.h;
        // Carry fully.
        for _ in 0..2 {
            h2 += h1 >> 44;
            h1 &= M44;
            h0 += (h2 >> 42) * 5;
            h2 &= M42;
            h1 += h0 >> 44;
            h0 &= M44;
        }
        h2 += h1 >> 44;
        h1 &= M44;
        // g = h + 5 - 2^130; keep g when it doesn't go negative.
        let mut g0 = h0 + 5;
        let mut g1 = h1 + (g0 >> 44);
        g0 &= M44;
        let g2 = (h2 + (g1 >> 44)).wrapping_sub(1 << 42);
        g1 &= M44;
        let keep_g = (g2 >> 63).wrapping_sub(1);
        h0 = (h0 & !keep_g) | (g0 & keep_g);
        h1 = (h1 & !keep_g) | (g1 & keep_g);
        h2 = (h2 & !keep_g) | (g2 & keep_g & M42);
        // h + s mod 2^128.
        let [p0, p1] = self.pad;
        h0 += p0 & M44;
        h1 += (((p0 >> 44) | (p1 << 20)) & M44) + (h0 >> 44);
        h0 &= M44;
        h2 += ((p1 >> 24) & M42) + (h1 >> 44);
        h1 &= M44;
        let lo = h0 | (h1 << 44);
        let hi = (h1 >> 20) | (h2 << 24);
        let mut out = [0u8; 16];
        out[..8].copy_from_slice(&lo.to_le_bytes());
        out[8..].copy_from_slice(&hi.to_le_bytes());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::super::hex;
    use super::*;

    fn tag(key: &[u8], msg: &[u8]) -> Vec<u8> {
        let mut p = Poly1305::new(key.try_into().unwrap());
        p.update(msg);
        p.finish().to_vec()
    }

    // RFC 8439 §2.5.2.
    #[test]
    fn rfc8439_tag() {
        let key = hex("85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b");
        assert_eq!(tag(&key, b"Cryptographic Forum Research Group"), hex("a8061dc1305136c6c22b8baf0c0127a9"));
    }

    // RFC 8439 Appendix A.3, vectors 1, 2 and the edge cases 7-11.
    #[test]
    fn rfc8439_appendix_a3() {
        assert_eq!(tag(&[0; 32], &[0; 64]), [0; 16]);
        let mut key = vec![0u8; 16];
        key.extend(hex("36e5f6b5c5e06070f0efca96227a863e"));
        let ietf = b"Any submission to the IETF intended by the Contributor for publication as all or part of an IETF Internet-Draft or RFC and any statement made within the context of an IETF activity is considered an \"IETF Contribution\". Such statements include oral statements in IETF sessions, as well as written and electronic communications made at any time or place, which are addressed to";
        assert_eq!(tag(&key, ietf), hex("36e5f6b5c5e06070f0efca96227a863e"));
        // h reaches p and must reduce to zero (vectors 6-11 exercise the
        // final carries).
        let mut k = hex("02000000000000000000000000000000");
        k.extend([0u8; 16]);
        assert_eq!(tag(&k, &[0xff; 16]), hex("03000000000000000000000000000000"));
        let mut k = hex("02000000000000000000000000000000");
        k.extend([0xffu8; 16]);
        assert_eq!(tag(&k, &hex("02000000000000000000000000000000")), hex("03000000000000000000000000000000"));
        let mut k = hex("01000000000000000000000000000000");
        k.extend([0u8; 16]);
        let m = hex("ffffffffffffffffffffffffffffffff f0ffffffffffffffffffffffffffffff 11000000000000000000000000000000");
        assert_eq!(tag(&k, &m), hex("05000000000000000000000000000000"));
        let m = hex("ffffffffffffffffffffffffffffffff fbfefefefefefefefefefefefefefefe 01010101010101010101010101010101");
        assert_eq!(tag(&k, &m), [0; 16]);
        let mut k = hex("02000000000000000000000000000000");
        k.extend([0u8; 16]);
        let m = hex("fdffffffffffffffffffffffffffffff");
        assert_eq!(tag(&k, &m), hex("faffffffffffffffffffffffffffffff"));
    }

    #[test]
    fn split_updates_agree() {
        let key: Vec<u8> = (1..=32).collect();
        let msg: Vec<u8> = (0..100u8).collect();
        let whole = tag(&key, &msg);
        for cut in [0, 1, 15, 16, 17, 99] {
            let mut p = Poly1305::new(key.as_slice().try_into().unwrap());
            p.update(&msg[..cut]);
            p.update(&msg[cut..]);
            assert_eq!(p.finish().to_vec(), whole, "cut {cut}");
        }
    }
}
