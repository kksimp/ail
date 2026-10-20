//! HMAC (RFC 2104) over any `Hash`, for HKDF and TLS Finished
//! (RFC 8446 §4.4.4).

use super::sha2::Hash;

/// An HMAC computation in progress.
#[derive(Clone)]
pub struct Hmac<H: Hash> {
    inner: H,
    outer: H,
}

impl<H: Hash> Hmac<H> {
    pub fn new(key: &[u8]) -> Self {
        let mut k = if key.len() > H::BLOCK { H::digest(key) } else { key.to_vec() };
        k.resize(H::BLOCK, 0);
        let mut inner = H::new();
        inner.update(&k.iter().map(|b| b ^ 0x36).collect::<Vec<_>>());
        let mut outer = H::new();
        outer.update(&k.iter().map(|b| b ^ 0x5c).collect::<Vec<_>>());
        Hmac { inner, outer }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    pub fn finish(self) -> Vec<u8> {
        let mut outer = self.outer;
        outer.update(&self.inner.finish());
        outer.finish()
    }

    /// The MAC of `data` under `key` in one call.
    pub fn mac(key: &[u8], data: &[u8]) -> Vec<u8> {
        let mut m = Self::new(key);
        m.update(data);
        m.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::super::hex;
    use super::super::sha2::{Sha256, Sha384, Sha512};
    use super::*;

    // RFC 4231 §4.2, §4.3, §4.7 (test cases 1, 2 and 6).
    #[test]
    fn rfc4231_vectors() {
        let k1 = [0x0bu8; 20];
        assert_eq!(
            Hmac::<Sha256>::mac(&k1, b"Hi There"),
            hex("b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7")
        );
        assert_eq!(
            Hmac::<Sha384>::mac(&k1, b"Hi There"),
            hex("afd03944d84895626b0825f4ab46907f15f9dadbe4101ec682aa034c7cebc59c
                 faea9ea9076ede7f4af152e8b2fa9cb6")
        );
        assert_eq!(
            Hmac::<Sha512>::mac(&k1, b"Hi There"),
            hex("87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cde
                 daa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854")
        );
        assert_eq!(
            Hmac::<Sha256>::mac(b"Jefe", b"what do ya want for nothing?"),
            hex("5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843")
        );
        // A key longer than the block is hashed first.
        let k6 = [0xaau8; 131];
        assert_eq!(
            Hmac::<Sha256>::mac(&k6, b"Test Using Larger Than Block-Size Key - Hash Key First"),
            hex("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54")
        );
    }
}
