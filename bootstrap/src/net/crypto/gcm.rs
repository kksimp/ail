//! AES-GCM (NIST SP 800-38D) with 96-bit nonces and 16-byte tags, for
//! TLS_AES_128_GCM_SHA256 (RFC 8446 §B.4). GHASH multiplies bit by bit
//! with masks, no tables (ADR 0366).

use super::BadTag;
use super::aes::Aes;
use super::ct;

/// GF(2^128) multiply in GCM's bit order (SP 800-38D §6.3, Algorithm 1),
/// branch-free in both operands.
fn gf_mul(x: u128, y: u128) -> u128 {
    const R: u128 = 0xe1 << 120;
    let mut z = 0u128;
    let mut v = y;
    for i in (0..128).rev() {
        z ^= v & 0u128.wrapping_sub((x >> i) & 1);
        v = (v >> 1) ^ (R & 0u128.wrapping_sub(v & 1));
    }
    z
}

fn ghash_update(h: u128, acc: &mut u128, data: &[u8]) {
    for chunk in data.chunks(16) {
        let mut b = [0u8; 16];
        b[..chunk.len()].copy_from_slice(chunk);
        *acc = gf_mul(*acc ^ u128::from_be_bytes(b), h);
    }
}

/// An AES-GCM key.
#[derive(Clone)]
pub struct AesGcm {
    aes: Aes,
    h: u128,
}

impl AesGcm {
    /// A 16- or 32-byte AES key.
    pub fn new(key: &[u8]) -> Self {
        let aes = Aes::new(key);
        let mut zero = [0u8; 16];
        aes.encrypt(&mut zero);
        AesGcm { aes, h: u128::from_be_bytes(zero) }
    }

    /// CTR mode from counter block `inc32(J0)` (SP 800-38D §6.5).
    fn ctr(&self, nonce: &[u8; 12], data: &mut [u8]) {
        let mut counter = [0u8; 16];
        counter[..12].copy_from_slice(nonce);
        for (i, chunk) in data.chunks_mut(16).enumerate() {
            counter[12..].copy_from_slice(&(i as u32).wrapping_add(2).to_be_bytes());
            let mut ks = counter;
            self.aes.encrypt(&mut ks);
            for (b, k) in chunk.iter_mut().zip(ks.iter()) {
                *b ^= k;
            }
        }
    }

    fn tag(&self, nonce: &[u8; 12], aad: &[u8], ct: &[u8]) -> [u8; 16] {
        let mut acc = 0u128;
        ghash_update(self.h, &mut acc, aad);
        ghash_update(self.h, &mut acc, ct);
        let lens = ((aad.len() as u128 * 8) << 64) | (ct.len() as u128 * 8);
        acc = gf_mul(acc ^ lens, self.h);
        let mut j0 = [0u8; 16];
        j0[..12].copy_from_slice(nonce);
        j0[15] = 1;
        self.aes.encrypt(&mut j0);
        (acc ^ u128::from_be_bytes(j0)).to_be_bytes()
    }

    /// Encrypts and returns ciphertext with the 16-byte tag appended.
    pub fn seal(&self, nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let mut out = plaintext.to_vec();
        self.ctr(nonce, &mut out);
        let t = self.tag(nonce, aad, &out);
        out.extend_from_slice(&t);
        out
    }

    /// Checks the tag in constant time, then decrypts.
    pub fn open(&self, nonce: &[u8; 12], aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>, BadTag> {
        if sealed.len() < 16 {
            return Err(BadTag);
        }
        let (ct_, given) = sealed.split_at(sealed.len() - 16);
        if !ct::eq(&self.tag(nonce, aad, ct_), given) {
            return Err(BadTag);
        }
        let mut out = ct_.to_vec();
        self.ctr(nonce, &mut out);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::super::hex;
    use super::*;

    fn n(s: &str) -> [u8; 12] {
        hex(s).try_into().unwrap()
    }

    // The GCM specification's test cases 1-4 (McGrew and Viega, as used
    // in NIST's GCMVS), AES-128.
    #[test]
    fn gcm_spec_cases_1_to_4() {
        let g = AesGcm::new(&[0; 16]);
        assert_eq!(g.seal(&[0; 12], b"", b""), hex("58e2fccefa7e3061367f1d57a4e7455a"));
        assert_eq!(
            g.seal(&[0; 12], b"", &[0; 16]),
            hex("0388dace60b6a392f328c2b971b2fe78 ab6e47d42cec13bdf53a67b21257bddf")
        );
        let g = AesGcm::new(&hex("feffe9928665731c6d6a8f9467308308"));
        let iv = n("cafebabefacedbaddecaf888");
        let p = hex(
            "d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a72
             1c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b391aafd255",
        );
        let c = hex(
            "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e
             21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091473f5985",
        );
        let mut want = c.clone();
        want.extend(hex("4d5c2af327cd64a62cf35abd2ba6fab4"));
        assert_eq!(g.seal(&iv, b"", &p), want);
        assert_eq!(g.open(&iv, b"", &want).unwrap(), p);
        let aad = hex("feedfacedeadbeeffeedfacedeadbeefabaddad2");
        let mut want = c[..60].to_vec();
        want.extend(hex("5bc94fbc3221a5db94fae95ae7121a47"));
        assert_eq!(g.seal(&iv, &aad, &p[..60]), want);
        assert_eq!(g.open(&iv, &aad, &want).unwrap(), &p[..60]);
    }

    // The same cases with AES-256 (test cases 13 and 14).
    #[test]
    fn gcm_spec_aes256() {
        let g = AesGcm::new(&[0; 32]);
        assert_eq!(g.seal(&[0; 12], b"", b""), hex("530f8afbc74536b9a963b4f1c4cb738b"));
        assert_eq!(
            g.seal(&[0; 12], b"", &[0; 16]),
            hex("cea7403d4d606b6e074ec5d3baf39d18 d0d1c8a799996bf0265b98b5d48ab919")
        );
    }

    #[test]
    fn tampering_fails() {
        let g = AesGcm::new(&[7; 16]);
        let sealed = g.seal(&[1; 12], b"head", b"some plaintext that spans two blocks");
        assert_eq!(g.open(&[1; 12], b"head", &sealed).unwrap(), b"some plaintext that spans two blocks");
        for i in [0, 20, sealed.len() - 1] {
            let mut bad = sealed.clone();
            bad[i] ^= 0x80;
            assert_eq!(g.open(&[1; 12], b"head", &bad), Err(BadTag));
        }
        assert_eq!(g.open(&[2; 12], b"head", &sealed), Err(BadTag));
        assert_eq!(g.open(&[1; 12], b"hea", &sealed), Err(BadTag));
    }
}
