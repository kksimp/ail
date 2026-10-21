//! Unsigned big integers and Montgomery arithmetic for *verifying*
//! signatures (RSA, ECDSA; ADR 0368). Every value here is public (keys,
//! signatures, hashes of public messages), so this code is not constant
//! time and must never handle a secret.

use std::cmp::Ordering;

/// A non-negative integer: little-endian 64-bit limbs, no high zero limbs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Big {
    limbs: Vec<u64>,
}

impl Big {
    pub fn zero() -> Big {
        Big { limbs: Vec::new() }
    }

    pub fn from_u64(v: u64) -> Big {
        Big { limbs: vec![v] }.norm()
    }

    fn norm(mut self) -> Big {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
        self
    }

    pub fn from_be(bytes: &[u8]) -> Big {
        let limbs = bytes
            .rchunks(8)
            .map(|c| {
                let mut b = [0u8; 8];
                b[8 - c.len()..].copy_from_slice(c);
                u64::from_be_bytes(b)
            })
            .collect();
        Big { limbs }.norm()
    }

    /// Big-endian bytes, left-padded to `len` (which must be enough).
    pub fn to_be(&self, len: usize) -> Vec<u8> {
        let mut out = vec![0u8; len];
        for (i, limb) in self.limbs.iter().enumerate() {
            for (j, b) in limb.to_le_bytes().iter().enumerate() {
                let k = 8 * i + j;
                if *b != 0 {
                    assert!(k < len, "number doesn't fit in {len} bytes");
                }
                if k < len {
                    out[len - 1 - k] = *b;
                }
            }
        }
        out
    }

    pub fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    pub fn bits(&self) -> usize {
        match self.limbs.last() {
            None => 0,
            Some(top) => 64 * self.limbs.len() - top.leading_zeros() as usize,
        }
    }

    pub fn bit(&self, i: usize) -> bool {
        self.limbs.get(i / 64).is_some_and(|l| (l >> (i % 64)) & 1 == 1)
    }

    pub fn is_odd(&self) -> bool {
        self.bit(0)
    }

    pub fn add(&self, o: &Big) -> Big {
        let n = self.limbs.len().max(o.limbs.len());
        let mut out = Vec::with_capacity(n + 1);
        let mut carry = 0u64;
        for i in 0..n {
            let (a, b) = (*self.limbs.get(i).unwrap_or(&0), *o.limbs.get(i).unwrap_or(&0));
            let (s1, c1) = a.overflowing_add(b);
            let (s2, c2) = s1.overflowing_add(carry);
            out.push(s2);
            carry = (c1 as u64) + (c2 as u64);
        }
        out.push(carry);
        Big { limbs: out }.norm()
    }

    /// `self - o`; `self >= o` is required.
    pub fn sub(&self, o: &Big) -> Big {
        assert!(*self >= *o, "Big::sub underflow");
        let mut out = Vec::with_capacity(self.limbs.len());
        let mut borrow = 0u64;
        for i in 0..self.limbs.len() {
            let b = *o.limbs.get(i).unwrap_or(&0);
            let (d1, c1) = self.limbs[i].overflowing_sub(b);
            let (d2, c2) = d1.overflowing_sub(borrow);
            out.push(d2);
            borrow = (c1 as u64) + (c2 as u64);
        }
        Big { limbs: out }.norm()
    }

    fn shl1(&self) -> Big {
        let mut out = Vec::with_capacity(self.limbs.len() + 1);
        let mut carry = 0;
        for &l in &self.limbs {
            out.push((l << 1) | carry);
            carry = l >> 63;
        }
        out.push(carry);
        Big { limbs: out }.norm()
    }

    /// `self mod m` by binary long division (used only to reduce inputs,
    /// not in loops).
    pub fn rem(&self, m: &Big) -> Big {
        assert!(!m.is_zero());
        let mut r = Big::zero();
        for i in (0..self.bits()).rev() {
            r = r.shl1();
            if self.bit(i) {
                r = r.add(&Big::from_u64(1));
            }
            if r >= *m {
                r = r.sub(m);
            }
        }
        r
    }
}

impl PartialOrd for Big {
    fn partial_cmp(&self, o: &Big) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Big {
    fn cmp(&self, o: &Big) -> Ordering {
        self.limbs.len().cmp(&o.limbs.len()).then_with(|| self.limbs.iter().rev().cmp(o.limbs.iter().rev()))
    }
}

/// Arithmetic modulo an odd `m` in Montgomery form, on fixed-width limbs.
#[derive(Clone)]
pub struct Mont {
    pub m: Big,
    n: usize,
    m_limbs: Vec<u64>,
    m_inv: u64, // -m^-1 mod 2^64
    r2: Vec<u64>,
}

/// A residue in Montgomery form (`n` limbs, value < m).
pub type Res = Vec<u64>;

impl Mont {
    pub fn new(m: &Big) -> Mont {
        assert!(m.is_odd(), "Montgomery modulus must be odd");
        let n = m.limbs.len();
        // Newton iteration for m^-1 mod 2^64.
        let m0 = m.limbs[0];
        let mut inv = 1u64;
        for _ in 0..6 {
            inv = inv.wrapping_mul(2u64.wrapping_sub(m0.wrapping_mul(inv)));
        }
        // R^2 mod m, R = 2^(64n), by doubling 1 2*64n times.
        let mut r2 = Big::from_u64(1);
        for _ in 0..128 * n {
            r2 = r2.shl1();
            if r2 >= *m {
                r2 = r2.sub(m);
            }
        }
        let mut ctx = Mont { m: m.clone(), n, m_limbs: m.limbs.clone(), m_inv: inv.wrapping_neg(), r2: Vec::new() };
        ctx.r2 = ctx.pad(&r2);
        ctx
    }

    fn pad(&self, x: &Big) -> Res {
        let mut v = x.limbs.clone();
        v.resize(self.n, 0);
        v
    }

    fn geq_m(&self, t: &[u64]) -> bool {
        for i in (0..self.n).rev() {
            if t[i] != self.m_limbs[i] {
                return t[i] > self.m_limbs[i];
            }
        }
        true
    }

    fn sub_m(&self, t: &mut [u64]) {
        let mut borrow = 0u64;
        for i in 0..self.n {
            let (d1, c1) = t[i].overflowing_sub(self.m_limbs[i]);
            let (d2, c2) = d1.overflowing_sub(borrow);
            t[i] = d2;
            borrow = (c1 as u64) + (c2 as u64);
        }
    }

    /// a * b * R^-1 mod m (CIOS).
    pub fn mul(&self, a: &Res, b: &Res) -> Res {
        let n = self.n;
        let mut t = vec![0u64; n + 2];
        for i in 0..n {
            let mut carry = 0u128;
            for j in 0..n {
                let s = t[j] as u128 + a[j] as u128 * b[i] as u128 + carry;
                t[j] = s as u64;
                carry = s >> 64;
            }
            let s = t[n] as u128 + carry;
            t[n] = s as u64;
            t[n + 1] = (s >> 64) as u64;
            let u = t[0].wrapping_mul(self.m_inv);
            let s = t[0] as u128 + u as u128 * self.m_limbs[0] as u128;
            let mut carry = s >> 64;
            for j in 1..n {
                let s = t[j] as u128 + u as u128 * self.m_limbs[j] as u128 + carry;
                t[j - 1] = s as u64;
                carry = s >> 64;
            }
            let s = t[n] as u128 + carry;
            t[n - 1] = s as u64;
            t[n] = t[n + 1] + (s >> 64) as u64;
        }
        let mut out = t[..n].to_vec();
        if t[n] != 0 || self.geq_m(&out) {
            self.sub_m(&mut out);
        }
        out
    }

    /// Into Montgomery form; `x` is reduced first.
    pub fn to_mont(&self, x: &Big) -> Res {
        let x = if *x >= self.m { x.rem(&self.m) } else { x.clone() };
        self.mul(&self.pad(&x), &self.r2)
    }

    pub fn from_mont(&self, a: &Res) -> Big {
        let mut one = vec![0u64; self.n];
        one[0] = 1;
        Big { limbs: self.mul(a, &one) }.norm()
    }

    pub fn one(&self) -> Res {
        self.to_mont(&Big::from_u64(1))
    }

    pub fn add(&self, a: &Res, b: &Res) -> Res {
        let mut out = vec![0u64; self.n];
        let mut carry = 0u64;
        for i in 0..self.n {
            let (s1, c1) = a[i].overflowing_add(b[i]);
            let (s2, c2) = s1.overflowing_add(carry);
            out[i] = s2;
            carry = (c1 as u64) + (c2 as u64);
        }
        if carry != 0 || self.geq_m(&out) {
            self.sub_m(&mut out);
        }
        out
    }

    pub fn sub(&self, a: &Res, b: &Res) -> Res {
        let mut out = vec![0u64; self.n];
        let mut borrow = 0u64;
        for i in 0..self.n {
            let (d1, c1) = a[i].overflowing_sub(b[i]);
            let (d2, c2) = d1.overflowing_sub(borrow);
            out[i] = d2;
            borrow = (c1 as u64) + (c2 as u64);
        }
        if borrow != 0 {
            let mut carry = 0u64;
            for i in 0..self.n {
                let (s1, c1) = out[i].overflowing_add(self.m_limbs[i]);
                let (s2, c2) = s1.overflowing_add(carry);
                out[i] = s2;
                carry = (c1 as u64) + (c2 as u64);
            }
        }
        out
    }

    pub fn is_zero(&self, a: &Res) -> bool {
        a.iter().all(|&l| l == 0)
    }

    /// base^e in Montgomery form (public exponent: plain square-and-multiply).
    pub fn pow(&self, base: &Res, e: &Big) -> Res {
        let mut r = self.one();
        for i in (0..e.bits()).rev() {
            r = self.mul(&r, &r);
            if e.bit(i) {
                r = self.mul(&r, base);
            }
        }
        r
    }

    /// 1/a for a prime modulus (Fermat).
    pub fn inv_prime(&self, a: &Res) -> Res {
        self.pow(a, &self.m.sub(&Big::from_u64(2)))
    }

    /// (x^e) mod m for plain numbers.
    pub fn modpow(&self, x: &Big, e: &Big) -> Big {
        self.from_mont(&self.pow(&self.to_mont(x), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::crypto::hex;

    #[test]
    fn basic_arithmetic() {
        let a = Big::from_be(&hex("ffffffffffffffffffffffffffffffff01"));
        let b = Big::from_be(&hex("ff"));
        assert_eq!(a.add(&b).sub(&b), a);
        assert_eq!(a.to_be(17), hex("ffffffffffffffffffffffffffffffff01"));
        assert_eq!(Big::from_be(&hex("0000000001")), Big::from_u64(1));
        assert_eq!(Big::from_u64(1000).rem(&Big::from_u64(7)), Big::from_u64(6));
        assert!(Big::from_u64(5) < Big::from_be(&hex("010000000000000000")));
        assert_eq!(a.bits(), 136);
    }

    #[test]
    fn modpow_matches_small_cases() {
        let m = Mont::new(&Big::from_u64(1_000_000_007));
        assert_eq!(m.modpow(&Big::from_u64(2), &Big::from_u64(10)), Big::from_u64(1024));
        // Fermat: a^(p-1) = 1.
        assert_eq!(m.modpow(&Big::from_u64(123456), &Big::from_u64(1_000_000_006)), Big::from_u64(1));
        let a = m.to_mont(&Big::from_u64(98765));
        assert_eq!(m.from_mont(&m.mul(&a, &m.inv_prime(&a))), Big::from_u64(1));
        assert_eq!(
            m.from_mont(&m.sub(&m.to_mont(&Big::from_u64(3)), &m.to_mont(&Big::from_u64(5)))),
            Big::from_u64(1_000_000_005)
        );
    }

    // A 521-bit Mersenne prime modulus spanning nine limbs: (2^521 - 2)^2
    // = 1 mod p, and 3^(p-1) = 1.
    #[test]
    fn multi_limb_modulus() {
        let mut p_bytes = vec![0x01];
        p_bytes.extend([0xff; 65]);
        let p = Big::from_be(&p_bytes);
        assert_eq!(p.bits(), 521);
        let m = Mont::new(&p);
        let minus_one = p.sub(&Big::from_u64(1));
        assert_eq!(m.modpow(&minus_one, &Big::from_u64(2)), Big::from_u64(1));
        assert_eq!(m.modpow(&Big::from_u64(3), &minus_one), Big::from_u64(1));
    }
}
