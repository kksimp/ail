//! X.509 certificates for the TLS client (RFC 5280; ADR 0368): parsing,
//! signature checks, chain building to the platform's roots, and host name
//! matching. Verification only, for server authentication.

#![allow(dead_code)]

pub mod der;
pub mod roots;
pub mod verify;

use crate::net::crypto::bignum::Big;
use crate::net::crypto::sig::{self, Curve, HashAlg, RsaKey};
use der::{Bad, Der, R};

// Object identifiers, as DER contents.
const OID_RSA: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01];
const OID_RSA_SHA256: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0b];
const OID_RSA_SHA384: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0c];
const OID_RSA_SHA512: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0d];
const OID_RSA_PSS: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0a];
const OID_EC: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01];
const OID_P256: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07];
const OID_P384: &[u8] = &[0x2b, 0x81, 0x04, 0x00, 0x22];
const OID_ECDSA_SHA256: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x02];
const OID_ECDSA_SHA384: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x03];
const OID_ECDSA_SHA512: &[u8] = &[0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x04];
const OID_SHA256: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01];
const OID_SHA384: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x02];
const OID_SHA512: &[u8] = &[0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x03];
const OID_MGF1: &[u8] = &[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x08];
const OID_CN: &[u8] = &[0x55, 0x04, 0x03];
const OID_SAN: &[u8] = &[0x55, 0x1d, 0x11];
const OID_BASIC: &[u8] = &[0x55, 0x1d, 0x13];
const OID_KEY_USAGE: &[u8] = &[0x55, 0x1d, 0x0f];
const OID_EKU: &[u8] = &[0x55, 0x1d, 0x25];
const OID_NAME_CONSTRAINTS: &[u8] = &[0x55, 0x1d, 0x1e];
const OID_POLICIES: &[u8] = &[0x55, 0x1d, 0x20];
const OID_POLICY_CONSTRAINTS: &[u8] = &[0x55, 0x1d, 0x24];
const OID_INHIBIT_ANY: &[u8] = &[0x55, 0x1d, 0x36];
const OID_SERVER_AUTH: &[u8] = &[0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01];
const OID_ANY_EKU: &[u8] = &[0x55, 0x1d, 0x25, 0x00];

/// A subject public key this verifier can use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicKey {
    Rsa(RsaKey),
    Ec(Curve, Vec<u8>),
    /// A key of another type (Ed25519, DSA, another curve): parsed so the
    /// certificate can sit in a chain, but nothing it signed verifies.
    Unsupported(String),
}

/// How a certificate (or a TLS CertificateVerify) is signed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SigAlg {
    Pkcs1(HashAlg),
    Pss(HashAlg),
    Ecdsa(HashAlg),
}

/// A name constraint subtree (RFC 5280 §4.2.1.10), DNS and IP forms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Subtree {
    Dns(String),
    Ip(Vec<u8>, Vec<u8>),
    /// A form this verifier doesn't check (directory names, email).
    Other,
}

/// The parts of a certificate the verifier uses.
#[derive(Clone, Debug)]
pub struct Cert {
    pub der: Vec<u8>,
    tbs: Vec<u8>,
    sig_alg: Option<SigAlg>,
    sig_alg_name: String,
    signature: Vec<u8>,
    pub issuer: Vec<u8>,
    pub subject: Vec<u8>,
    pub not_before: i64,
    pub not_after: i64,
    pub key: PublicKey,
    pub is_ca: bool,
    pub path_len: Option<u64>,
    /// keyUsage bits (bit 0 = digitalSignature), when present.
    pub key_usage: Option<u16>,
    /// extKeyUsage OIDs, when present.
    pub eku: Option<Vec<Vec<u8>>>,
    pub dns_names: Vec<String>,
    pub ip_addrs: Vec<Vec<u8>>,
    pub permitted: Vec<Subtree>,
    pub excluded: Vec<Subtree>,
}

impl Cert {
    /// Parses a DER certificate. Unknown critical extensions are refused
    /// (RFC 5280 §4.2).
    pub fn parse(der: &[u8]) -> Result<Cert, String> {
        parse(der).map_err(|Bad(why)| format!("a certificate is malformed ({why})"))
    }

    /// The subject's common name, or its whole name, for messages.
    pub fn display_name(&self) -> String {
        name_cn(&self.subject).unwrap_or_else(|| "(unnamed)".to_string())
    }

    pub fn issuer_name(&self) -> String {
        name_cn(&self.issuer).unwrap_or_else(|| "(unnamed)".to_string())
    }

    /// Whether `issuer_key` signed this certificate.
    pub fn signed_by(&self, issuer_key: &PublicKey) -> Result<(), String> {
        let Some(alg) = self.sig_alg else {
            return Err(format!("`{}` is signed with {}, which AIL's TLS doesn't support", self.display_name(), self.sig_alg_name));
        };
        if verify_signature(issuer_key, alg, &self.tbs, &self.signature) {
            Ok(())
        } else {
            Err(format!("the signature on `{}` doesn't verify", self.display_name()))
        }
    }

    pub fn self_issued(&self) -> bool {
        self.issuer == self.subject
    }
}

/// Verifies `signature` over `msg` with `key` and `alg`.
pub fn verify_signature(key: &PublicKey, alg: SigAlg, msg: &[u8], signature: &[u8]) -> bool {
    match (key, alg) {
        (PublicKey::Rsa(k), SigAlg::Pkcs1(h)) => k.verify_pkcs1(h, msg, signature),
        (PublicKey::Rsa(k), SigAlg::Pss(h)) => k.verify_pss(h, msg, signature),
        (PublicKey::Ec(curve, point), SigAlg::Ecdsa(h)) => {
            let Ok((r, s)) = ecdsa_sig(signature) else { return false };
            sig::ecdsa_verify(*curve, point, &h.digest(msg), r, s)
        }
        _ => false,
    }
}

/// An ECDSA-Sig-Value: SEQUENCE { r INTEGER, s INTEGER }.
fn ecdsa_sig(der: &[u8]) -> R<(&[u8], &[u8])> {
    let mut outer = Der::new(der);
    let mut seq = outer.nested(der::SEQUENCE)?;
    outer.finish()?;
    let r = seq.uint()?;
    let s = seq.uint()?;
    seq.finish()?;
    Ok((r, s))
}

fn hash_oid(oid: &[u8]) -> Option<HashAlg> {
    match oid {
        OID_SHA256 => Some(HashAlg::Sha256),
        OID_SHA384 => Some(HashAlg::Sha384),
        OID_SHA512 => Some(HashAlg::Sha512),
        _ => None,
    }
}

/// An AlgorithmIdentifier for a signature, or a printable name for one
/// this verifier lacks.
fn sig_alg(body: &[u8]) -> R<Result<SigAlg, String>> {
    let mut d = Der::new(body);
    let oid = d.expect(der::OID)?;
    let alg = match oid {
        OID_RSA_SHA256 | OID_RSA_SHA384 | OID_RSA_SHA512 => {
            // Parameters must be NULL (or, tolerated, absent).
            if !d.is_empty() {
                d.expect(der::NULL)?;
            }
            Ok(SigAlg::Pkcs1(match oid {
                OID_RSA_SHA256 => HashAlg::Sha256,
                OID_RSA_SHA384 => HashAlg::Sha384,
                _ => HashAlg::Sha512,
            }))
        }
        OID_ECDSA_SHA256 => Ok(SigAlg::Ecdsa(HashAlg::Sha256)),
        OID_ECDSA_SHA384 => Ok(SigAlg::Ecdsa(HashAlg::Sha384)),
        OID_ECDSA_SHA512 => Ok(SigAlg::Ecdsa(HashAlg::Sha512)),
        OID_RSA_PSS => pss_params(&mut d)?.map(SigAlg::Pss).ok_or_else(|| "RSA-PSS with unusual parameters".to_string()),
        // Unknown (SHA-1, MD5, Ed25519, ...): its parameters aren't ours to
        // check. Roots signed this way still serve as trust anchors.
        _ => {
            let name = match oid_text(oid).as_str() {
                "1.2.840.113549.1.1.5" => "SHA-1 with RSA".to_string(),
                "1.2.840.113549.1.1.4" => "MD5 with RSA".to_string(),
                "1.2.840.10045.4.1" => "ECDSA with SHA-1".to_string(),
                "1.3.101.112" => "Ed25519".to_string(),
                other => format!("the algorithm {other}"),
            };
            return Ok(Err(name));
        }
    };
    d.finish()?;
    Ok(alg)
}

/// RSASSA-PSS-params (RFC 4055 §3.1): accepted when the hash and MGF1
/// hash agree and the salt is the hash length (the only form in use).
fn pss_params(d: &mut Der) -> R<Option<HashAlg>> {
    let mut p = d.nested(der::SEQUENCE)?;
    let mut hash = None;
    let mut mgf_hash = None;
    let mut salt = 20u64;
    if let Some(h) = p.optional(der::ctx(0))? {
        let mut a = Der::new(h).nested(der::SEQUENCE)?;
        hash = hash_oid(a.expect(der::OID)?);
    }
    if let Some(m) = p.optional(der::ctx(1))? {
        let mut a = Der::new(m).nested(der::SEQUENCE)?;
        if a.expect(der::OID)? != OID_MGF1 {
            return Ok(None);
        }
        let mut inner = a.nested(der::SEQUENCE)?;
        mgf_hash = hash_oid(inner.expect(der::OID)?);
    }
    if let Some(s) = p.optional(der::ctx(2))? {
        let v = Der::new(s).uint()?;
        if v.len() > 2 {
            return Ok(None);
        }
        salt = v.iter().fold(0u64, |a, &b| a << 8 | b as u64);
    }
    // trailerField [3], if present, must be 1; ignore the rest.
    Ok(match (hash, mgf_hash) {
        (Some(h), Some(m)) if h == m && salt == h.len() as u64 => Some(h),
        _ => None,
    })
}

fn oid_text(oid: &[u8]) -> String {
    let mut parts = Vec::new();
    if let Some(&first) = oid.first() {
        parts.push((first / 40).min(2) as u64);
        parts.push(first as u64 - 40 * parts[0]);
    }
    let mut v = 0u64;
    for &b in oid.iter().skip(1) {
        v = (v << 7) | (b & 0x7f) as u64;
        if b & 0x80 == 0 {
            parts.push(v);
            v = 0;
        }
    }
    parts.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(".")
}

fn public_key(spki: &[u8]) -> R<PublicKey> {
    let mut d = Der::new(spki);
    let mut alg = d.nested(der::SEQUENCE)?;
    let key_bits = d.bit_string_bytes()?;
    d.finish()?;
    let oid = alg.expect(der::OID)?;
    Ok(match oid {
        OID_RSA => {
            let mut k = Der::new(key_bits).nested(der::SEQUENCE)?;
            let n = Big::from_be(k.uint()?);
            let e = Big::from_be(k.uint()?);
            k.finish()?;
            PublicKey::Rsa(RsaKey { n, e })
        }
        OID_EC => {
            let curve = match alg.expect(der::OID)? {
                OID_P256 => Some(Curve::P256),
                OID_P384 => Some(Curve::P384),
                _ => None,
            };
            match curve {
                Some(c) => PublicKey::Ec(c, key_bits.to_vec()),
                None => PublicKey::Unsupported("an elliptic curve other than P-256 and P-384".into()),
            }
        }
        _ => PublicKey::Unsupported(format!("key type {}", oid_text(oid))),
    })
}

/// The first commonName in a Name, if it's a string.
fn name_cn(name: &[u8]) -> Option<String> {
    let mut rdns = Der::new(name).nested(der::SEQUENCE).ok()?;
    while !rdns.is_empty() {
        let mut set = rdns.nested(der::SET).ok()?;
        while !set.is_empty() {
            let mut atv = set.nested(der::SEQUENCE).ok()?;
            if atv.expect(der::OID).ok()? == OID_CN {
                let (_, value, _) = atv.any().ok()?;
                return Some(String::from_utf8_lossy(value).into_owned());
            }
        }
    }
    None
}

fn parse(der_bytes: &[u8]) -> R<Cert> {
    let mut outer = Der::new(der_bytes);
    let mut cert = outer.nested(der::SEQUENCE)?;
    outer.finish()?;
    let (tbs_body, tbs_whole) = cert.expect_whole(der::SEQUENCE)?;
    let outer_alg = cert.expect(der::SEQUENCE)?;
    let signature = cert.bit_string_bytes()?;
    cert.finish()?;

    let mut t = Der::new(tbs_body);
    if let Some(v) = t.optional(der::ctx(0))? {
        let mut v = Der::new(v);
        let version = v.uint()?;
        v.finish()?;
        if version != [2] && version != [1] && version != [0] {
            return Err(Bad("unknown certificate version"));
        }
    }
    t.uint()?; // serialNumber
    let inner_alg = t.expect(der::SEQUENCE)?;
    if inner_alg != outer_alg {
        return Err(Bad("the two signature algorithms differ"));
    }
    let (_, issuer) = t.expect_whole(der::SEQUENCE)?;
    let mut validity = t.nested(der::SEQUENCE)?;
    let (tag, nb, _) = validity.any()?;
    let not_before = der::time(tag, nb)?;
    let (tag, na, _) = validity.any()?;
    let not_after = der::time(tag, na)?;
    validity.finish()?;
    let (_, subject) = t.expect_whole(der::SEQUENCE)?;
    let spki = t.expect(der::SEQUENCE)?;
    let key = public_key(spki)?;
    t.optional(der::ctx_prim(1))?; // issuerUniqueID
    t.optional(der::ctx_prim(2))?; // subjectUniqueID
    let mut c = Cert {
        der: der_bytes.to_vec(),
        tbs: tbs_whole.to_vec(),
        sig_alg: None,
        sig_alg_name: String::new(),
        signature: signature.to_vec(),
        issuer: issuer.to_vec(),
        subject: subject.to_vec(),
        not_before,
        not_after,
        key,
        is_ca: false,
        path_len: None,
        key_usage: None,
        eku: None,
        dns_names: Vec::new(),
        ip_addrs: Vec::new(),
        permitted: Vec::new(),
        excluded: Vec::new(),
    };
    match sig_alg(outer_alg)? {
        Ok(a) => c.sig_alg = Some(a),
        Err(name) => c.sig_alg_name = name,
    }
    if let Some(exts) = t.optional(der::ctx(3))? {
        let mut list = Der::new(exts).nested(der::SEQUENCE)?;
        let mut seen: Vec<&[u8]> = Vec::new();
        while !list.is_empty() {
            let mut e = list.nested(der::SEQUENCE)?;
            let oid = e.expect(der::OID)?;
            let critical = if e.peek() == Some(der::BOOLEAN) { e.boolean()? } else { false };
            let value = e.expect(der::OCTET_STRING)?;
            e.finish()?;
            if seen.contains(&oid) {
                return Err(Bad("an extension appears twice"));
            }
            seen.push(oid);
            extension(&mut c, oid, critical, value)?;
        }
    }
    t.finish()?;
    Ok(c)
}

fn extension(c: &mut Cert, oid: &[u8], critical: bool, value: &[u8]) -> R<()> {
    let mut d = Der::new(value);
    match oid {
        OID_BASIC => {
            let mut s = d.nested(der::SEQUENCE)?;
            if s.peek() == Some(der::BOOLEAN) {
                c.is_ca = s.boolean()?;
            }
            if !s.is_empty() {
                let v = s.uint()?;
                if v.len() > 4 {
                    return Err(Bad("path length too large"));
                }
                c.path_len = Some(v.iter().fold(0u64, |a, &b| a << 8 | b as u64));
            }
            s.finish()?;
        }
        OID_KEY_USAGE => {
            let b = d.expect(der::BIT_STRING)?;
            let (&unused, bits) = b.split_first().ok_or(Bad("empty key usage"))?;
            if unused > 7 || bits.len() > 2 {
                return Err(Bad("bad key usage"));
            }
            // Bit 0 is the first byte's top bit.
            let mut v = 0u16;
            for (i, byte) in bits.iter().enumerate() {
                for j in 0..8 {
                    if byte & (0x80 >> j) != 0 {
                        v |= 1 << (8 * i + j);
                    }
                }
            }
            c.key_usage = Some(v);
        }
        OID_EKU => {
            let mut s = d.nested(der::SEQUENCE)?;
            let mut list = Vec::new();
            while !s.is_empty() {
                list.push(s.expect(der::OID)?.to_vec());
            }
            c.eku = Some(list);
        }
        OID_SAN => {
            let mut s = d.nested(der::SEQUENCE)?;
            while !s.is_empty() {
                let (tag, body, _) = s.any()?;
                match tag {
                    0x82 => c.dns_names.push(String::from_utf8(body.to_vec()).map_err(|_| Bad("non-ASCII DNS name"))?),
                    0x87 if body.len() == 4 || body.len() == 16 => c.ip_addrs.push(body.to_vec()),
                    0x87 => return Err(Bad("bad IP address in subjectAltName")),
                    _ => {} // email, URI, directory names: not used for servers
                }
            }
        }
        OID_NAME_CONSTRAINTS => {
            let mut s = d.nested(der::SEQUENCE)?;
            if let Some(p) = s.optional(der::ctx(0))? {
                c.permitted = subtrees(p)?;
            }
            if let Some(x) = s.optional(der::ctx(1))? {
                c.excluded = subtrees(x)?;
            }
            s.finish()?;
        }
        // Accepted without processing, as browsers do: no policy checks.
        OID_POLICIES | OID_POLICY_CONSTRAINTS | OID_INHIBIT_ANY => return Ok(()),
        _ if critical => return Err(Bad("an unknown critical extension")),
        _ => return Ok(()),
    }
    d.finish()
}

fn subtrees(body: &[u8]) -> R<Vec<Subtree>> {
    let mut d = Der::new(body);
    let mut out = Vec::new();
    while !d.is_empty() {
        let mut st = d.nested(der::SEQUENCE)?;
        let (tag, base, _) = st.any()?;
        // minimum/maximum [0]/[1] are never used on the web; ignore them.
        out.push(match tag {
            0x82 => Subtree::Dns(String::from_utf8_lossy(base).to_ascii_lowercase()),
            0x87 if base.len() == 8 || base.len() == 32 => {
                let half = base.len() / 2;
                Subtree::Ip(base[..half].to_vec(), base[half..].to_vec())
            }
            _ => Subtree::Other,
        });
    }
    Ok(out)
}

/// Certificates from PEM text (`-----BEGIN CERTIFICATE-----` blocks).
pub fn pem_certs(text: &str) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut b64 = String::new();
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line == "-----BEGIN CERTIFICATE-----" {
            inside = true;
            b64.clear();
        } else if line == "-----END CERTIFICATE-----" {
            if inside && let Some(der) = base64(&b64) {
                out.push(der);
            }
            inside = false;
        } else if inside {
            b64.push_str(line);
        }
    }
    out
}

fn base64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut acc = 0u32;
    let mut bits = 0;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\t' | b'\r' | b'\n' => continue,
            _ => return None,
        };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
pub mod tests;
