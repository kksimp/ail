//! HKDF (RFC 5869) and TLS 1.3's `HKDF-Expand-Label` (RFC 8446 §7.1).

use super::hmac::Hmac;
use super::sha2::Hash;

/// HKDF-Extract (RFC 5869 §2.2): the pseudorandom key from `salt` and
/// input keying material `ikm`. An empty salt means `Hash.OUT` zero bytes.
pub fn extract<H: Hash>(salt: &[u8], ikm: &[u8]) -> Vec<u8> {
    let zeros;
    let salt = if salt.is_empty() {
        zeros = vec![0u8; H::OUT];
        &zeros[..]
    } else {
        salt
    };
    Hmac::<H>::mac(salt, ikm)
}

/// HKDF-Expand (RFC 5869 §2.3): `len` bytes from `prk` and `info`.
/// `len` is at most 255 times the hash length.
pub fn expand<H: Hash>(prk: &[u8], info: &[u8], len: usize) -> Vec<u8> {
    assert!(len <= 255 * H::OUT, "HKDF-Expand length too large");
    let mut out = Vec::with_capacity(len);
    let mut t: Vec<u8> = Vec::new();
    let mut i = 1u8;
    while out.len() < len {
        let mut m = Hmac::<H>::new(prk);
        m.update(&t);
        m.update(info);
        m.update(&[i]);
        t = m.finish();
        out.extend_from_slice(&t);
        i = i.wrapping_add(1);
    }
    out.truncate(len);
    out
}

/// HKDF-Expand-Label (RFC 8446 §7.1): the label gets the prefix `tls13 `.
pub fn expand_label<H: Hash>(secret: &[u8], label: &str, context: &[u8], len: usize) -> Vec<u8> {
    let full = format!("tls13 {label}");
    assert!(full.len() <= 255 && context.len() <= 255 && len <= 0xffff);
    let mut info = Vec::with_capacity(4 + full.len() + context.len());
    info.extend_from_slice(&(len as u16).to_be_bytes());
    info.push(full.len() as u8);
    info.extend_from_slice(full.as_bytes());
    info.push(context.len() as u8);
    info.extend_from_slice(context);
    expand::<H>(secret, &info, len)
}

/// Derive-Secret (RFC 8446 §7.1): `expand_label` over a transcript hash.
pub fn derive_secret<H: Hash>(secret: &[u8], label: &str, transcript_hash: &[u8]) -> Vec<u8> {
    expand_label::<H>(secret, label, transcript_hash, H::OUT)
}

#[cfg(test)]
mod tests {
    use super::super::hex;
    use super::super::sha2::Sha256;
    use super::*;

    // RFC 5869 Appendix A.1.
    #[test]
    fn rfc5869_case_1() {
        let ikm = [0x0bu8; 22];
        let salt = hex("000102030405060708090a0b0c");
        let info = hex("f0f1f2f3f4f5f6f7f8f9");
        let prk = extract::<Sha256>(&salt, &ikm);
        assert_eq!(prk, hex("077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5"));
        assert_eq!(
            expand::<Sha256>(&prk, &info, 42),
            hex("3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf
                 34007208d5b887185865")
        );
    }

    // RFC 5869 Appendix A.2: longer inputs, 82 bytes out.
    #[test]
    fn rfc5869_case_2() {
        let ikm: Vec<u8> = (0x00..=0x4f).collect();
        let salt: Vec<u8> = (0x60..=0xaf).collect();
        let info: Vec<u8> = (0xb0..=0xff).collect();
        let prk = extract::<Sha256>(&salt, &ikm);
        assert_eq!(prk, hex("06a6b88c5853361a06104c9ceb35b45cef760014904671014a193f40c15fc244"));
        assert_eq!(
            expand::<Sha256>(&prk, &info, 82),
            hex("b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c
                 59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71
                 cc30c58179ec3e87c14c01d5c1f3434f1d87")
        );
    }

    // RFC 5869 Appendix A.3: empty salt and info.
    #[test]
    fn rfc5869_case_3() {
        let ikm = [0x0bu8; 22];
        let prk = extract::<Sha256>(&[], &ikm);
        assert_eq!(prk, hex("19ef24a32c717b167f33a91d6f648bdf96596776afdb6377ac434c1c293ccb04"));
        assert_eq!(
            expand::<Sha256>(&prk, &[], 42),
            hex("8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d
                 9d201395faa4b61a96c8")
        );
    }

    // RFC 8448 §3: the early secret and its `derived` secret with no PSK.
    #[test]
    fn rfc8448_early_secret() {
        let early = extract::<Sha256>(&[], &[0u8; 32]);
        assert_eq!(early, hex("33ad0a1c607ec03b09e6cd9893680ce210adf300aa1f2660e1b22e10f170f92a"));
        let empty_hash = Sha256::digest(b"");
        assert_eq!(
            derive_secret::<Sha256>(&early, "derived", &empty_hash),
            hex("6f2615a108c702c5678f54fc9dbab69716c076189c48250cebeac3576c3611ba")
        );
    }
}
