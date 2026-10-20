//! ChaCha20 and the ChaCha20-Poly1305 AEAD (RFC 8439 §2.3–2.8), for the
//! TLS_CHACHA20_POLY1305_SHA256 suite (RFC 8446 §B.4). Add-rotate-xor
//! only, so constant time by construction (ADR 0366).

use super::BadTag;
use super::ct;
use super::poly1305::Poly1305;

#[inline(always)]
fn quarter(s: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(16);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(12);
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(8);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(7);
}

/// The ChaCha20 block function (RFC 8439 §2.3): 64 bytes of key stream.
pub fn block(key: &[u8; 32], counter: u32, nonce: &[u8; 12]) -> [u8; 64] {
    let mut init = [0u32; 16];
    init[..4].copy_from_slice(&[0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]);
    for i in 0..8 {
        init[4 + i] = u32::from_le_bytes(key[4 * i..4 * i + 4].try_into().unwrap());
    }
    init[12] = counter;
    for i in 0..3 {
        init[13 + i] = u32::from_le_bytes(nonce[4 * i..4 * i + 4].try_into().unwrap());
    }
    let mut s = init;
    for _ in 0..10 {
        quarter(&mut s, 0, 4, 8, 12);
        quarter(&mut s, 1, 5, 9, 13);
        quarter(&mut s, 2, 6, 10, 14);
        quarter(&mut s, 3, 7, 11, 15);
        quarter(&mut s, 0, 5, 10, 15);
        quarter(&mut s, 1, 6, 11, 12);
        quarter(&mut s, 2, 7, 8, 13);
        quarter(&mut s, 3, 4, 9, 14);
    }
    let mut out = [0u8; 64];
    for i in 0..16 {
        out[4 * i..4 * i + 4].copy_from_slice(&s[i].wrapping_add(init[i]).to_le_bytes());
    }
    out
}

/// XORs the key stream starting at block `counter` into `data`
/// (RFC 8439 §2.4).
pub fn xor(key: &[u8; 32], counter: u32, nonce: &[u8; 12], data: &mut [u8]) {
    for (i, chunk) in data.chunks_mut(64).enumerate() {
        let ks = block(key, counter.wrapping_add(i as u32), nonce);
        for (b, k) in chunk.iter_mut().zip(ks.iter()) {
            *b ^= k;
        }
    }
}

/// The AEAD's tag over `aad` and ciphertext `ct` (RFC 8439 §2.8).
fn tag(key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], ct: &[u8]) -> [u8; 16] {
    let otk: [u8; 32] = block(key, 0, nonce)[..32].try_into().unwrap();
    let mut p = Poly1305::new(&otk);
    let zeros = [0u8; 16];
    p.update(aad);
    p.update(&zeros[..(16 - aad.len() % 16) % 16]);
    p.update(ct);
    p.update(&zeros[..(16 - ct.len() % 16) % 16]);
    p.update(&(aad.len() as u64).to_le_bytes());
    p.update(&(ct.len() as u64).to_le_bytes());
    p.finish()
}

/// ChaCha20-Poly1305 encryption (RFC 8439 §2.8): returns ciphertext with
/// the 16-byte tag appended.
pub fn seal(key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let mut out = plaintext.to_vec();
    xor(key, 1, nonce, &mut out);
    let t = tag(key, nonce, aad, &out);
    out.extend_from_slice(&t);
    out
}

/// ChaCha20-Poly1305 decryption: checks the tag (in constant time) before
/// decrypting, and returns nothing on a mismatch.
pub fn open(key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>, BadTag> {
    if sealed.len() < 16 {
        return Err(BadTag);
    }
    let (ct_, given) = sealed.split_at(sealed.len() - 16);
    if !ct::eq(&tag(key, nonce, aad, ct_), given) {
        return Err(BadTag);
    }
    let mut out = ct_.to_vec();
    xor(key, 1, nonce, &mut out);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::hex;
    use super::*;

    const SUNSCREEN: &[u8] = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";

    fn key_0_to_31() -> [u8; 32] {
        std::array::from_fn(|i| i as u8)
    }

    // RFC 8439 §2.3.2.
    #[test]
    fn rfc8439_block() {
        let nonce: [u8; 12] = hex("000000090000004a00000000").try_into().unwrap();
        assert_eq!(
            block(&key_0_to_31(), 1, &nonce).to_vec(),
            hex("10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4e
                 d2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e")
        );
    }

    // RFC 8439 §2.4.2.
    #[test]
    fn rfc8439_encryption() {
        let nonce: [u8; 12] = hex("000000000000004a00000000").try_into().unwrap();
        let mut data = SUNSCREEN.to_vec();
        xor(&key_0_to_31(), 1, &nonce, &mut data);
        assert_eq!(
            data,
            hex("6e2e359a2568f98041ba0728dd0d6981e97e7aec1d4360c20a27afccfd9fae0b
                 f91b65c5524733ab8f593dabcd62b3571639d624e65152ab8f530c359f0861d8
                 07ca0dbf500d6a6156a38e088a22b65e52bc514d16ccf806818ce91ab7793736
                 5af90bbf74a35be6b40b8eedf2785e42874d")
        );
    }

    // RFC 8439 §2.6.2: the Poly1305 one-time key is block 0's first half.
    #[test]
    fn rfc8439_one_time_key() {
        let key: [u8; 32] = hex("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
            .try_into()
            .unwrap();
        let nonce: [u8; 12] = hex("000000000001020304050607").try_into().unwrap();
        assert_eq!(
            block(&key, 0, &nonce)[..32].to_vec(),
            hex("8ad5a08b905f81cc815040274ab29471a833b637e3fd0da508dbb8e2fdd1a646")
        );
    }

    // RFC 8439 §2.8.2.
    #[test]
    fn rfc8439_aead() {
        let key: [u8; 32] = hex("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
            .try_into()
            .unwrap();
        let nonce: [u8; 12] = hex("070000004041424344454647").try_into().unwrap();
        let aad = hex("50515253c0c1c2c3c4c5c6c7");
        let sealed = seal(&key, &nonce, &aad, SUNSCREEN);
        assert_eq!(
            sealed,
            hex("d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d6
                 3dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b36
                 92ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc
                 3ff4def08e4b7a9de576d26586cec64b6116
                 1ae10b594f09e26a7e902ecbd0600691")
        );
        assert_eq!(open(&key, &nonce, &aad, &sealed).unwrap(), SUNSCREEN);
        // Any flipped bit, in the ciphertext, the tag or the AAD, fails.
        for i in [0, 50, sealed.len() - 1] {
            let mut bad = sealed.clone();
            bad[i] ^= 1;
            assert_eq!(open(&key, &nonce, &aad, &bad), Err(BadTag));
        }
        assert_eq!(open(&key, &nonce, b"", &sealed), Err(BadTag));
        assert_eq!(open(&key, &nonce, &aad, &sealed[..15]), Err(BadTag));
    }
}
