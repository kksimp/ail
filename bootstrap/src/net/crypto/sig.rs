//! Signature verification for certificates and TLS 1.3 CertificateVerify
//! (ADR 0368): ECDSA over P-256 and P-384 (FIPS 186-4, SEC 1), RSA PKCS#1
//! v1.5 and RSASSA-PSS (RFC 8017). Verification only: every input is
//! public, so `bignum`'s variable-time arithmetic is acceptable here.

use super::bignum::{Big, Mont, Res};
use super::sha2::{Hash, Sha256, Sha384, Sha512};

/// A hash named by a signature algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashAlg {
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlg {
    pub fn digest(self, data: &[u8]) -> Vec<u8> {
        match self {
            HashAlg::Sha256 => Sha256::digest(data),
            HashAlg::Sha384 => Sha384::digest(data),
            HashAlg::Sha512 => Sha512::digest(data),
        }
    }

    pub fn len(self) -> usize {
        match self {
            HashAlg::Sha256 => 32,
            HashAlg::Sha384 => 48,
            HashAlg::Sha512 => 64,
        }
    }

    /// The DER DigestInfo prefix for PKCS#1 v1.5 (RFC 8017 §9.2 note 1).
    fn digest_info(self) -> &'static [u8] {
        match self {
            HashAlg::Sha256 => &[
                0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05, 0x00,
                0x04, 0x20,
            ],
            HashAlg::Sha384 => &[
                0x30, 0x41, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02, 0x05, 0x00,
                0x04, 0x30,
            ],
            HashAlg::Sha512 => &[
                0x30, 0x51, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03, 0x05, 0x00,
                0x04, 0x40,
            ],
        }
    }
}

/// A NIST prime curve y^2 = x^3 - 3x + b.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curve {
    P256,
    P384,
}

struct CurveParams {
    p: &'static str,
    b: &'static str,
    gx: &'static str,
    gy: &'static str,
    n: &'static str,
}

impl Curve {
    fn params(self) -> CurveParams {
        match self {
            // FIPS 186-4 §D.1.2.3.
            Curve::P256 => CurveParams {
                p: "ffffffff00000001000000000000000000000000ffffffffffffffffffffffff",
                b: "5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b",
                gx: "6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296",
                gy: "4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5",
                n: "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551",
            },
            // FIPS 186-4 §D.1.2.4.
            Curve::P384 => CurveParams {
                p: "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff",
                b: "b3312fa7e23ee7e4988e056be3f82d19181d9c6efe8141120314088f5013875ac656398d8a2ed19d2a85c8edd3ec2aef",
                gx: "aa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab7",
                gy: "3617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f",
                n: "ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973",
            },
        }
    }

    /// Field element size in bytes.
    pub fn size(self) -> usize {
        match self {
            Curve::P256 => 32,
            Curve::P384 => 48,
        }
    }
}

fn big_hex(s: &str) -> Big {
    let bytes: Vec<u8> = (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect();
    Big::from_be(&bytes)
}

/// A point in Jacobian coordinates over the field, in Montgomery form;
/// Z = 0 is the point at infinity.
#[derive(Clone)]
struct Point {
    x: Res,
    y: Res,
    z: Res,
}

struct Group {
    f: Mont,
    b: Res,
    n: Big,
    g: Point,
}

impl Group {
    fn new(curve: Curve) -> Group {
        let c = curve.params();
        let f = Mont::new(&big_hex(c.p));
        let g = Point { x: f.to_mont(&big_hex(c.gx)), y: f.to_mont(&big_hex(c.gy)), z: f.one() };
        Group { b: f.to_mont(&big_hex(c.b)), n: big_hex(c.n), g, f }
    }

    fn infinity(&self) -> Point {
        let zero = self.f.sub(&self.f.one(), &self.f.one());
        Point { x: self.f.one(), y: self.f.one(), z: zero }
    }

    /// Whether affine (x, y) satisfies the curve equation.
    fn on_curve(&self, x: &Res, y: &Res) -> bool {
        let f = &self.f;
        let x3 = f.mul(&f.mul(x, x), x);
        let three_x = f.add(&f.add(x, x), x);
        let rhs = f.add(&f.sub(&x3, &three_x), &self.b);
        f.mul(y, y) == rhs
    }

    // dbl-2001-b (a = -3).
    fn double(&self, p: &Point) -> Point {
        let f = &self.f;
        if f.is_zero(&p.z) {
            return p.clone();
        }
        let delta = f.mul(&p.z, &p.z);
        let gamma = f.mul(&p.y, &p.y);
        let beta = f.mul(&p.x, &gamma);
        let t = f.mul(&f.sub(&p.x, &delta), &f.add(&p.x, &delta));
        let alpha = f.add(&f.add(&t, &t), &t);
        let beta4 = f.add(&f.add(&beta, &beta), &f.add(&beta, &beta));
        let beta8 = f.add(&beta4, &beta4);
        let x3 = f.sub(&f.mul(&alpha, &alpha), &beta8);
        let yz = f.add(&p.y, &p.z);
        let z3 = f.sub(&f.sub(&f.mul(&yz, &yz), &gamma), &delta);
        let g2 = f.mul(&gamma, &gamma);
        let g4 = f.add(&g2, &g2);
        let g8 = f.add(&f.add(&g4, &g4), &f.add(&g4, &g4));
        let y3 = f.sub(&f.mul(&alpha, &f.sub(&beta4, &x3)), &g8);
        Point { x: x3, y: y3, z: z3 }
    }

    // add-2007-bl, with the doubling and inverse cases.
    fn add(&self, p: &Point, q: &Point) -> Point {
        let f = &self.f;
        if f.is_zero(&p.z) {
            return q.clone();
        }
        if f.is_zero(&q.z) {
            return p.clone();
        }
        let z1z1 = f.mul(&p.z, &p.z);
        let z2z2 = f.mul(&q.z, &q.z);
        let u1 = f.mul(&p.x, &z2z2);
        let u2 = f.mul(&q.x, &z1z1);
        let s1 = f.mul(&f.mul(&p.y, &q.z), &z2z2);
        let s2 = f.mul(&f.mul(&q.y, &p.z), &z1z1);
        let h = f.sub(&u2, &u1);
        let r0 = f.sub(&s2, &s1);
        if f.is_zero(&h) {
            return if f.is_zero(&r0) { self.double(p) } else { self.infinity() };
        }
        let h2 = f.add(&h, &h);
        let i = f.mul(&h2, &h2);
        let j = f.mul(&h, &i);
        let r = f.add(&r0, &r0);
        let v = f.mul(&u1, &i);
        let x3 = f.sub(&f.sub(&f.mul(&r, &r), &j), &f.add(&v, &v));
        let s1j = f.mul(&s1, &j);
        let y3 = f.sub(&f.mul(&r, &f.sub(&v, &x3)), &f.add(&s1j, &s1j));
        let zz = f.add(&p.z, &q.z);
        let z3 = f.mul(&f.sub(&f.sub(&f.mul(&zz, &zz), &z1z1), &z2z2), &h);
        Point { x: x3, y: y3, z: z3 }
    }

    /// u1*G + u2*Q by interleaved double-and-add (Shamir's trick). The
    /// scalars are public in verification.
    fn double_mul(&self, u1: &Big, u2: &Big, q: &Point) -> Point {
        let gq = self.add(&self.g, q);
        let mut acc = self.infinity();
        for i in (0..u1.bits().max(u2.bits())).rev() {
            acc = self.double(&acc);
            match (u1.bit(i), u2.bit(i)) {
                (true, true) => acc = self.add(&acc, &gq),
                (true, false) => acc = self.add(&acc, &self.g),
                (false, true) => acc = self.add(&acc, q),
                _ => {}
            }
        }
        acc
    }

    /// The affine x coordinate as a number, or `None` at infinity.
    fn affine_x(&self, p: &Point) -> Option<Big> {
        let f = &self.f;
        if f.is_zero(&p.z) {
            return None;
        }
        let zi = f.inv_prime(&p.z);
        Some(f.from_mont(&f.mul(&p.x, &f.mul(&zi, &zi))))
    }
}

/// The hash as an integer, truncated to the bit length of n (SEC 1 §4.1.4).
fn hash_to_int(hash: &[u8], n: &Big) -> Big {
    let e = Big::from_be(hash);
    let excess = (8 * hash.len()).saturating_sub(n.bits());
    if excess == 0 {
        return e;
    }
    // Shift right by `excess` bits.
    let mut bytes = hash.to_vec();
    let mut carry = 0u8;
    let (shift_bytes, shift_bits) = (excess / 8, excess % 8);
    bytes.truncate(bytes.len() - shift_bytes);
    if shift_bits > 0 {
        for b in bytes.iter_mut() {
            let next = *b << (8 - shift_bits);
            *b = (*b >> shift_bits) | carry;
            carry = next;
        }
    }
    Big::from_be(&bytes)
}

/// ECDSA verification (SEC 1 §4.1.4): `public` is the uncompressed point
/// `04 || x || y`, `r` and `s` the signature's integers (big-endian).
pub fn ecdsa_verify(curve: Curve, public: &[u8], hash: &[u8], r: &[u8], s: &[u8]) -> bool {
    let size = curve.size();
    if public.len() != 1 + 2 * size || public[0] != 4 {
        return false;
    }
    let g = Group::new(curve);
    let (x, y) = (Big::from_be(&public[1..1 + size]), Big::from_be(&public[1 + size..]));
    if x >= g.f.m || y >= g.f.m {
        return false;
    }
    let (qx, qy) = (g.f.to_mont(&x), g.f.to_mont(&y));
    if !g.on_curve(&qx, &qy) {
        return false;
    }
    let q = Point { x: qx, y: qy, z: g.f.one() };
    let (r, s) = (Big::from_be(r), Big::from_be(s));
    if r.is_zero() || s.is_zero() || r >= g.n || s >= g.n {
        return false;
    }
    let nm = Mont::new(&g.n);
    let w = nm.inv_prime(&nm.to_mont(&s));
    let e = nm.to_mont(&hash_to_int(hash, &g.n));
    let u1 = nm.from_mont(&nm.mul(&e, &w));
    let u2 = nm.from_mont(&nm.mul(&nm.to_mont(&r), &w));
    match g.affine_x(&g.double_mul(&u1, &u2, &q)) {
        Some(x) => x.rem(&g.n) == r,
        None => false,
    }
}

/// An RSA public key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsaKey {
    pub n: Big,
    pub e: Big,
}

/// The smallest RSA modulus accepted (NIST SP 800-131A).
pub const RSA_MIN_BITS: usize = 2048;

impl RsaKey {
    /// s^e mod n as `k` bytes, or `None` for a malformed signature or key.
    fn public_op(&self, sig: &[u8]) -> Option<Vec<u8>> {
        let k = self.n.bits().div_ceil(8);
        if self.n.bits() < RSA_MIN_BITS || self.n.bits() > 8192 || !self.n.is_odd() || sig.len() != k {
            return None;
        }
        let s = Big::from_be(sig);
        if s >= self.n {
            return None;
        }
        Some(Mont::new(&self.n).modpow(&s, &self.e).to_be(k))
    }

    /// RSASSA-PKCS1-v1_5 verification (RFC 8017 §8.2.2).
    pub fn verify_pkcs1(&self, alg: HashAlg, msg: &[u8], sig: &[u8]) -> bool {
        let Some(em) = self.public_op(sig) else { return false };
        let t: Vec<u8> = [alg.digest_info(), &alg.digest(msg)].concat();
        let k = em.len();
        if k < t.len() + 11 {
            return false;
        }
        let mut want = vec![0x00, 0x01];
        want.resize(k - t.len() - 1, 0xff);
        want.push(0);
        want.extend_from_slice(&t);
        em == want
    }

    /// RSASSA-PSS verification (RFC 8017 §8.1.2, §9.1.2) with MGF1 over the
    /// same hash and a salt as long as the hash (RFC 8446 §4.2.3).
    pub fn verify_pss(&self, alg: HashAlg, msg: &[u8], sig: &[u8]) -> bool {
        let Some(em_full) = self.public_op(sig) else { return false };
        let h_len = alg.len();
        let em_bits = self.n.bits() - 1;
        let em_len = em_bits.div_ceil(8);
        // When the modulus is a whole number of bytes plus one bit, EM is
        // one byte shorter than k and the extra leading byte must be zero.
        let (lead, em) = em_full.split_at(em_full.len() - em_len);
        if lead.iter().any(|&b| b != 0) || em_len < 2 * h_len + 2 || em[em_len - 1] != 0xbc {
            return false;
        }
        let (masked_db, rest) = em.split_at(em_len - h_len - 1);
        let h = &rest[..h_len];
        let top_bits = 8 * em_len - em_bits;
        let top_mask = 0xffu8 >> top_bits;
        if masked_db[0] & !top_mask != 0 {
            return false;
        }
        let mut db = mgf1(alg, h, masked_db.len());
        for (d, m) in db.iter_mut().zip(masked_db) {
            *d ^= m;
        }
        db[0] &= top_mask;
        let ps_len = em_len - 2 * h_len - 2;
        if db[..ps_len].iter().any(|&b| b != 0) || db[ps_len] != 1 {
            return false;
        }
        let salt = &db[ps_len + 1..];
        let mut m_prime = vec![0u8; 8];
        m_prime.extend_from_slice(&alg.digest(msg));
        m_prime.extend_from_slice(salt);
        alg.digest(&m_prime) == h
    }
}

/// MGF1 (RFC 8017 §B.2.1).
fn mgf1(alg: HashAlg, seed: &[u8], len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(len + alg.len());
    let mut counter = 0u32;
    while out.len() < len {
        out.extend_from_slice(&alg.digest(&[seed, &counter.to_be_bytes()].concat()));
        counter += 1;
    }
    out.truncate(len);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::crypto::hex;

    #[test]
    fn curve_constants_are_consistent() {
        for curve in [Curve::P256, Curve::P384] {
            let g = Group::new(curve);
            assert!(g.on_curve(&g.g.x, &g.g.y), "{curve:?}: G is on the curve");
            // n*G is the point at infinity, and (n-1)*G is -G.
            let nm1 = g.n.sub(&Big::from_u64(1));
            let p = g.double_mul(&nm1, &Big::zero(), &g.infinity());
            assert_eq!(g.affine_x(&p), Some(g.f.from_mont(&g.g.x)), "{curve:?}");
            let inf = g.add(&p, &g.g);
            assert!(g.f.is_zero(&inf.z), "{curve:?}: n*G = O");
        }
    }

    #[test]
    fn hash_truncation() {
        let n = Big::from_be(&hex("ff")); // 8 bits
        assert_eq!(hash_to_int(&hex("abcd"), &n), Big::from_be(&hex("ab")));
        let n = Big::from_be(&hex("0fff")); // 12 bits
        assert_eq!(hash_to_int(&hex("abcd"), &n), Big::from_be(&hex("0abc")));
    }

    /// Splits a DER ECDSA-Sig-Value into r and s (tests only; the X.509
    /// code has the real parser).
    fn der_sig(sig: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
        let mut i = 2;
        let mut int = || {
            if *sig.get(i)? != 2 {
                return None;
            }
            let len = *sig.get(i + 1)? as usize;
            let v = sig.get(i + 2..i + 2 + len)?.to_vec();
            i += 2 + len;
            Some(v)
        };
        let r = int()?;
        Some((r, int()?))
    }

    // Signatures made by OpenSSL; every valid one verifies and every
    // altered one fails.
    #[test]
    fn openssl_signature_vectors() {
        let mut count = 0;
        for line in include_str!("testdata/signatures.txt").lines().filter(|l| !l.starts_with('#')) {
            let f: Vec<&str> = line.split(' ').collect();
            let (name, public, msg, sig, valid) = (f[0], f[1], f[2], hex(f[3]), f[4] == "1");
            let msg = if msg == "-" { vec![] } else { hex(msg) };
            let alg = if name.ends_with("sha256") {
                HashAlg::Sha256
            } else if name.ends_with("sha384") {
                HashAlg::Sha384
            } else {
                HashAlg::Sha512
            };
            let ok = if let Some(rest) = name.strip_prefix("ecdsa_") {
                let curve = if rest.starts_with("p256") { Curve::P256 } else { Curve::P384 };
                // A corrupted DER signature may not parse; that's a failure.
                match der_sig(&sig) {
                    Some((r, s)) => ecdsa_verify(curve, &hex(public), &alg.digest(&msg), &r, &s),
                    None => false,
                }
            } else {
                let (n, e) = public.split_once(':').unwrap();
                let key = RsaKey { n: Big::from_be(&hex(n)), e: Big::from_be(&hex(e)) };
                if name.starts_with("rsa_pss") {
                    key.verify_pss(alg, &msg, &sig)
                } else {
                    key.verify_pkcs1(alg, &msg, &sig)
                }
            };
            assert_eq!(ok, valid, "{name}: {line:.80}");
            count += 1;
        }
        assert!(count > 100);
    }
}
