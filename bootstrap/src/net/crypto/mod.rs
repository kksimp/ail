//! Primitive causes: std.tls.4 (crypto primitives), surfaced only as TLS failures.
//! Cryptographic primitives for the in-house TLS 1.3 client (ADR 0360,
//! ADR 0366). Rust standard library only. Each primitive is tested against
//! the known-answer vectors of the document that defines it.
//!
//! Constant time: where an input is secret (keys, plaintext, shared
//! secrets) no branch and no memory index depends on it. AES uses a
//! bitsliced S-box instead of tables, GHASH and Poly1305 multiply without
//! tables, and X25519 swaps with masks. Tag checks use `ct::eq`.
//!
//! Nothing here is used until the handshake (slice 2) lands, so the module
//! allows dead code for now.

#![allow(dead_code)]

pub mod aes;
pub mod bignum;
pub mod chacha20;
pub mod ct;
pub mod gcm;
pub mod hkdf;
pub mod hmac;
pub mod poly1305;
pub mod rand;
pub mod sha2;
pub mod sig;
pub mod x25519;

/// An AEAD failure: the tag didn't match, so nothing was decrypted.
#[derive(Debug, PartialEq, Eq)]
pub struct BadTag;

/// Decodes hex for the tests; whitespace is ignored.
#[cfg(test)]
pub fn hex(s: &str) -> Vec<u8> {
    let s: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    assert!(s.len() % 2 == 0, "odd hex length");
    s.chunks(2).map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap()).collect()
}

/// Every primitive against random inputs whose outputs came from OpenSSL
/// (`testdata/crosscheck.txt`), covering lengths around block edges.
#[cfg(test)]
mod crosscheck {
    use super::sha2::{Hash, Sha256, Sha384, Sha512};
    use super::*;

    #[test]
    fn matches_openssl() {
        let mut seen = 0;
        for line in include_str!("testdata/crosscheck.txt").lines() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let mut f = line.split(' ');
            let name = f.next().unwrap();
            let v: Vec<Vec<u8>> = f.map(|s| if s == "-" { vec![] } else { hex(s) }).collect();
            let got = match name {
                "sha256" => Sha256::digest(&v[0]),
                "sha384" => Sha384::digest(&v[0]),
                "sha512" => Sha512::digest(&v[0]),
                "hmac256" => hmac::Hmac::<Sha256>::mac(&v[0], &v[1]),
                "poly1305" => {
                    let mut p = poly1305::Poly1305::new(v[0].as_slice().try_into().unwrap());
                    p.update(&v[1]);
                    p.finish().to_vec()
                }
                "chacha20poly1305" => {
                    let (k, n) = (v[0].as_slice().try_into().unwrap(), v[1].as_slice().try_into().unwrap());
                    let sealed = chacha20::seal(k, n, &v[2], &v[3]);
                    assert_eq!(chacha20::open(k, n, &v[2], &sealed).unwrap(), v[3]);
                    sealed
                }
                "aesgcm" => {
                    let g = gcm::AesGcm::new(&v[0]);
                    let n = v[1].as_slice().try_into().unwrap();
                    let sealed = g.seal(n, &v[2], &v[3]);
                    assert_eq!(g.open(n, &v[2], &sealed).unwrap(), v[3]);
                    sealed
                }
                "hkdf256" => {
                    let len = u16::from_be_bytes([v[3][0], v[3][1]]) as usize;
                    hkdf::expand::<Sha256>(&hkdf::extract::<Sha256>(&v[0], &v[1]), &v[2], len)
                }
                "x25519" => x25519::x25519(
                    v[0].as_slice().try_into().unwrap(),
                    v[1].as_slice().try_into().unwrap(),
                )
                .to_vec(),
                other => panic!("unknown vector `{other}`"),
            };
            assert_eq!(&got, v.last().unwrap(), "{line}");
            seen += 1;
        }
        assert!(seen > 150);
    }
}
