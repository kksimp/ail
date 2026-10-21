//! Cipher suites, the transcript hash and the key schedule
//! (RFC 8446 §4.4.1, §7.1–7.3, §B.4).

use crate::net::crypto::gcm::AesGcm;
use crate::net::crypto::hkdf;
use crate::net::crypto::hmac::Hmac;
use crate::net::crypto::sha2::{Hash, Sha256, Sha384};
use crate::net::crypto::{BadTag, chacha20};

/// A TLS 1.3 cipher suite this client offers, most preferred first in
/// `Suite::OFFERED`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suite {
    Chacha20Poly1305Sha256,
    Aes128GcmSha256,
    Aes256GcmSha384,
}

impl Suite {
    /// ChaCha20 first: it is constant time and fast in software, while
    /// AES-GCM here is the slower portable path (ADR 0366).
    pub const OFFERED: [Suite; 3] = [Suite::Chacha20Poly1305Sha256, Suite::Aes128GcmSha256, Suite::Aes256GcmSha384];

    pub fn code(self) -> u16 {
        match self {
            Suite::Aes128GcmSha256 => 0x1301,
            Suite::Aes256GcmSha384 => 0x1302,
            Suite::Chacha20Poly1305Sha256 => 0x1303,
        }
    }

    pub fn from_code(code: u16) -> Option<Suite> {
        Suite::OFFERED.into_iter().find(|s| s.code() == code)
    }

    pub fn name(self) -> &'static str {
        match self {
            Suite::Aes128GcmSha256 => "TLS_AES_128_GCM_SHA256",
            Suite::Aes256GcmSha384 => "TLS_AES_256_GCM_SHA384",
            Suite::Chacha20Poly1305Sha256 => "TLS_CHACHA20_POLY1305_SHA256",
        }
    }

    fn sha384(self) -> bool {
        self == Suite::Aes256GcmSha384
    }

    pub fn hash_len(self) -> usize {
        if self.sha384() { 48 } else { 32 }
    }

    fn key_len(self) -> usize {
        if self == Suite::Aes128GcmSha256 { 16 } else { 32 }
    }

    pub fn hash(self, data: &[u8]) -> Vec<u8> {
        if self.sha384() { Sha384::digest(data) } else { Sha256::digest(data) }
    }

    pub fn extract(self, salt: &[u8], ikm: &[u8]) -> Vec<u8> {
        if self.sha384() { hkdf::extract::<Sha384>(salt, ikm) } else { hkdf::extract::<Sha256>(salt, ikm) }
    }

    pub fn expand_label(self, secret: &[u8], label: &str, context: &[u8], len: usize) -> Vec<u8> {
        if self.sha384() {
            hkdf::expand_label::<Sha384>(secret, label, context, len)
        } else {
            hkdf::expand_label::<Sha256>(secret, label, context, len)
        }
    }

    pub fn derive_secret(self, secret: &[u8], label: &str, transcript_hash: &[u8]) -> Vec<u8> {
        self.expand_label(secret, label, transcript_hash, self.hash_len())
    }

    pub fn hmac(self, key: &[u8], data: &[u8]) -> Vec<u8> {
        if self.sha384() { Hmac::<Sha384>::mac(key, data) } else { Hmac::<Sha256>::mac(key, data) }
    }

    /// Finished's `verify_data` (RFC 8446 §4.4.4).
    pub fn finished(self, base_key: &[u8], transcript_hash: &[u8]) -> Vec<u8> {
        let finished_key = self.expand_label(base_key, "finished", b"", self.hash_len());
        self.hmac(&finished_key, transcript_hash)
    }
}

/// The running handshake transcript (RFC 8446 §4.4.1). It keeps the
/// messages, since the suite (and so the hash) is known only after
/// ServerHello; handshakes are a few kilobytes.
#[derive(Default)]
pub struct Transcript {
    messages: Vec<u8>,
}

impl Transcript {
    pub fn add(&mut self, msg: &[u8]) {
        self.messages.extend_from_slice(msg);
    }

    pub fn hash(&self, suite: Suite) -> Vec<u8> {
        suite.hash(&self.messages)
    }

    /// After a HelloRetryRequest, ClientHello1 is replaced by a synthetic
    /// `message_hash` message holding its hash (RFC 8446 §4.4.1).
    pub fn restart_after_retry(&mut self, suite: Suite) {
        let h = self.hash(suite);
        let mut m = vec![254, 0, 0, h.len() as u8];
        m.extend_from_slice(&h);
        self.messages = m;
    }
}

/// The secrets of one connection (RFC 8446 §7.1).
pub struct Secrets {
    pub client_handshake: Vec<u8>,
    pub server_handshake: Vec<u8>,
    master: Vec<u8>,
}

impl Secrets {
    /// From the (EC)DHE shared secret and the transcript hash through
    /// ServerHello. No PSK, so the early secret has zero input.
    pub fn handshake(suite: Suite, shared: &[u8], hello_hash: &[u8]) -> Secrets {
        let zeros = vec![0u8; suite.hash_len()];
        let empty = suite.hash(b"");
        let early = suite.extract(&[], &zeros);
        let hs = suite.extract(&suite.derive_secret(&early, "derived", &empty), shared);
        Secrets {
            client_handshake: suite.derive_secret(&hs, "c hs traffic", hello_hash),
            server_handshake: suite.derive_secret(&hs, "s hs traffic", hello_hash),
            master: suite.extract(&suite.derive_secret(&hs, "derived", &empty), &zeros),
        }
    }

    /// Client and server application traffic secrets, from the transcript
    /// hash through the server's Finished.
    pub fn application(&self, suite: Suite, finished_hash: &[u8]) -> (Vec<u8>, Vec<u8>) {
        (
            suite.derive_secret(&self.master, "c ap traffic", finished_hash),
            suite.derive_secret(&self.master, "s ap traffic", finished_hash),
        )
    }
}

enum Aead {
    Chacha([u8; 32]),
    Gcm(Box<AesGcm>),
}

/// One direction's record protection: AEAD key, IV and sequence number
/// (RFC 8446 §5.2–5.3, §7.3).
pub struct TrafficKeys {
    suite: Suite,
    secret: Vec<u8>,
    aead: Aead,
    iv: [u8; 12],
    seq: u64,
}

impl TrafficKeys {
    pub fn new(suite: Suite, secret: &[u8]) -> TrafficKeys {
        let key = suite.expand_label(secret, "key", b"", suite.key_len());
        let iv = suite.expand_label(secret, "iv", b"", 12).try_into().unwrap();
        let aead = match suite {
            Suite::Chacha20Poly1305Sha256 => Aead::Chacha(key.try_into().unwrap()),
            _ => Aead::Gcm(Box::new(AesGcm::new(&key))),
        };
        TrafficKeys { suite, secret: secret.to_vec(), aead, iv, seq: 0 }
    }

    /// The keys after a KeyUpdate (RFC 8446 §7.2).
    pub fn next(&self) -> TrafficKeys {
        let s = self.suite.expand_label(&self.secret, "traffic upd", b"", self.suite.hash_len());
        TrafficKeys::new(self.suite, &s)
    }

    /// The per-record nonce: the IV xor the sequence number (§5.3).
    fn nonce(&mut self) -> [u8; 12] {
        let mut n = self.iv;
        for (b, s) in n[4..].iter_mut().zip(self.seq.to_be_bytes()) {
            *b ^= s;
        }
        // 2^64 records can't happen within a connection's lifetime; refuse
        // rather than wrap if it ever did.
        self.seq = self.seq.checked_add(1).expect("TLS sequence number exhausted");
        n
    }

    pub fn seal(&mut self, aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let nonce = self.nonce();
        match &self.aead {
            Aead::Chacha(k) => chacha20::seal(k, &nonce, aad, plaintext),
            Aead::Gcm(g) => g.seal(&nonce, aad, plaintext),
        }
    }

    pub fn open(&mut self, aad: &[u8], sealed: &[u8]) -> Result<Vec<u8>, BadTag> {
        let nonce = self.nonce();
        match &self.aead {
            Aead::Chacha(k) => chacha20::open(k, &nonce, aad, sealed),
            Aead::Gcm(g) => g.open(&nonce, aad, sealed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::crypto::hex;

    const S: Suite = Suite::Aes128GcmSha256;

    // RFC 8448 §3 values, as transcribed in Mbed TLS's
    // tests/suites/test_suite_ssl.data (Apache-2.0).
    #[test]
    fn rfc8448_handshake_derivations() {
        let hs = hex("005cb112fd8eb4ccc623bb88a07c64b3ede1605363fc7d0df8c7ce4ff0fb4ae6");
        let hello = hex("f736cb34fe25e701551bee6fd24c1cc7102a7daf9405cb15d97aafe16f757d03");
        assert_eq!(
            S.derive_secret(&hs, "c hs traffic", &hello),
            hex("2faac08f851d35fea3604fcb4de82dc62c9b164a70974d0462e27f1ab278700f")
        );
        assert_eq!(
            S.derive_secret(&hs, "s hs traffic", &hello),
            hex("fe927ae271312e8bf0275b581c54eef020450dc4ecffaa05a1a35d27518e7803")
        );
        let master = hex("e2d32d4ed66dd37897a0e80c84107503ce58bf8aad4cb55a5002d77ecb890ece");
        let fin = hex("b0aeffc46a2cfe33114e6fd7d51f9f04b1ca3c497dab08934a774a9d9ad7dbf3");
        assert_eq!(
            S.derive_secret(&master, "c ap traffic", &fin),
            hex("2abbf2b8e381d23dbebe1dd2a7d16a8bf484cb4950d23fb7fb7fa8547062d9a1")
        );
        assert_eq!(
            S.derive_secret(&master, "s ap traffic", &fin),
            hex("cc21f1bf8feb7dd5fa505bd9c4b468a9984d554a993dc49e6d285598fb672691")
        );
        // finished_key from the client handshake traffic secret.
        assert_eq!(
            S.expand_label(
                &hex("2faac08f851d35fea3604fcb4de82dc62c9b164a70974d0462e27f1ab278700f"),
                "finished",
                b"",
                32
            ),
            hex("5ace394c26980d581243f627d1150ae27e37fa52364e0a7f20ac686d09cd0e8e")
        );
        let shs = hex("b67b7d690cc16c4e75e54213cb2d37b4e9c912bcded9105d42befd59d391ad38");
        assert_eq!(S.expand_label(&shs, "key", b"", 16), hex("3fce516009c21727d0f2e4e86ee403bc"));
        assert_eq!(S.expand_label(&shs, "iv", b"", 12), hex("5d313eb2671276ee13000b30"));
    }

    // RFC 8448 §3: the client's first application data record, from the
    // client application key and IV.
    #[test]
    fn rfc8448_record_protection() {
        let mut k = TrafficKeys {
            suite: S,
            secret: vec![],
            aead: Aead::Gcm(Box::new(AesGcm::new(&hex("17422dda596ed5d9acd890e3c63f5051")))),
            iv: hex("5b78923dee08579033e523d9").try_into().unwrap(),
            seq: 0,
        };
        let mut inner: Vec<u8> = (0..0x32).collect();
        inner.push(23);
        let header = hex("1703030043");
        assert_eq!(
            k.seal(&header, &inner),
            hex("a23f7054b62c94d0affafe8228ba55cbefacea42f914aa66bcab3f2b9819a8a5
                 b46b395bd54a9a20441e2b62974e1f5a6292a2977014bd1e3deae63aeebb21694915e4")
        );
    }
}
