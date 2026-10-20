//! Server certificate verification (RFC 5280 §6, RFC 6125; ADR 0368):
//! a path from the leaf through the server's intermediates to a trusted
//! root, validity dates, key usages, name constraints, the host name, and
//! the TLS 1.3 CertificateVerify signature.

use super::{Cert, OID_ANY_EKU, OID_SERVER_AUTH, PublicKey, SigAlg, Subtree, der, verify_signature};
use crate::net::crypto::sig::{Curve, HashAlg};
use crate::net::tls::{TlsError, Verifier};

/// The longest chain searched (leaf, intermediates, root).
const MAX_DEPTH: usize = 8;

/// keyUsage bits (RFC 5280 §4.2.1.3).
const DIGITAL_SIGNATURE: u16 = 1 << 0;
const KEY_CERT_SIGN: u16 = 1 << 5;

/// Verifies servers against a set of trusted roots.
pub struct WebVerifier<'a> {
    pub roots: &'a [Cert],
    /// Seconds since 1970; `None` means the system clock.
    pub now: Option<i64>,
}

impl Verifier for WebVerifier<'_> {
    fn verify(&self, host: &str, chain: &[Vec<u8>], scheme: u16, signed: &[u8], signature: &[u8])
    -> Result<(), TlsError> {
        let now = self.now.unwrap_or_else(|| {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
        });
        let leaf = verify_chain(chain, self.roots, host, now).map_err(TlsError::certificate)?;
        let alg = match scheme {
            0x0403 | 0x0503 => {
                let want = if scheme == 0x0403 { Curve::P256 } else { Curve::P384 };
                match &leaf.key {
                    PublicKey::Ec(c, _) if *c == want => {}
                    _ => {
                        return Err(TlsError::certificate(format!(
                            "the server signed with scheme {scheme:#06x}, which doesn't match its certificate's key"
                        )));
                    }
                }
                SigAlg::Ecdsa(if scheme == 0x0403 { HashAlg::Sha256 } else { HashAlg::Sha384 })
            }
            0x0804 => SigAlg::Pss(HashAlg::Sha256),
            0x0805 => SigAlg::Pss(HashAlg::Sha384),
            0x0806 => SigAlg::Pss(HashAlg::Sha512),
            _ => return Err(TlsError::protocol(format!("the server used signature scheme {scheme:#06x}"))),
        };
        if !verify_signature(&leaf.key, alg, signed, signature) {
            return Err(TlsError::certificate(
                "the server's handshake signature doesn't match its certificate (decrypt_error)",
            ));
        }
        Ok(())
    }
}

/// Checks `chain` (leaf first, DER) for `host` at time `now`, returning
/// the parsed leaf.
pub fn verify_chain(chain: &[Vec<u8>], roots: &[Cert], host: &str, now: i64) -> Result<Cert, String> {
    let leaf = Cert::parse(chain.first().ok_or("the server sent no certificate")?)?;
    // Intermediates that don't parse are ignored: servers sometimes send
    // extras, and a needed one that's missing fails the path search.
    let pool: Vec<Cert> = chain[1..].iter().filter_map(|d| Cert::parse(d).ok()).collect();
    check_dates(&leaf, now)?;
    if let Some(eku) = &leaf.eku
        && !eku.iter().any(|o| o == OID_SERVER_AUTH || o == OID_ANY_EKU)
    {
        return Err(format!("the certificate for `{}` isn't for servers (extended key usage)", leaf.display_name()));
    }
    if let Some(ku) = leaf.key_usage
        && ku & DIGITAL_SIGNATURE == 0
    {
        return Err(format!("the certificate for `{}` doesn't allow signing (key usage)", leaf.display_name()));
    }
    check_host(&leaf, host)?;
    let mut search = Search { pool: &pool, roots, now, used: vec![false; pool.len()], best: None };
    let mut path = vec![&leaf];
    if search.extend(&mut path) {
        return Ok(leaf.clone());
    }
    Err(search.best.unwrap_or_else(|| {
        format!(
            "the certificate for `{}` doesn't lead to a trusted root: it was issued by `{}`, which isn't in the system's root store",
            leaf.display_name(),
            leaf.issuer_name()
        )
    }))
}

fn check_dates(c: &Cert, now: i64) -> Result<(), String> {
    if now < c.not_before {
        return Err(format!("the certificate for `{}` isn't valid until {}", c.display_name(), der::date(c.not_before)));
    }
    if now > c.not_after {
        return Err(format!("the certificate for `{}` expired on {}", c.display_name(), der::date(c.not_after)));
    }
    Ok(())
}

struct Search<'a> {
    pool: &'a [Cert],
    roots: &'a [Cert],
    now: i64,
    used: Vec<bool>,
    /// The most specific reason a candidate path failed.
    best: Option<String>,
}

impl<'a> Search<'a> {
    /// Tries to finish `path` (leaf first) at a root, depth first.
    fn extend(&mut self, path: &mut Vec<&'a Cert>) -> bool {
        let c = *path.last().unwrap();
        // The server may send the root itself.
        if path.len() > 1 && self.roots.iter().any(|r| r.der == c.der) {
            return self.constraints_hold(path, None);
        }
        for root in self.roots {
            if root.subject != c.issuer {
                continue;
            }
            match c.signed_by(&root.key) {
                Ok(()) if self.constraints_hold(path, Some(root)) => return true,
                Ok(()) => {}
                Err(e) => self.note(e),
            }
        }
        if path.len() >= MAX_DEPTH {
            self.note(format!("the certificate chain for `{}` is too long", path[0].display_name()));
            return false;
        }
        for i in 0..self.pool.len() {
            let cand = &self.pool[i];
            if self.used[i] || cand.subject != c.issuer || path.iter().any(|p| p.der == cand.der) {
                continue;
            }
            if let Err(e) = self.usable_ca(cand, path) {
                self.note(e);
                continue;
            }
            if let Err(e) = c.signed_by(&cand.key) {
                self.note(e);
                continue;
            }
            self.used[i] = true;
            path.push(cand);
            if self.extend(path) {
                return true;
            }
            path.pop();
            self.used[i] = false;
        }
        false
    }

    fn note(&mut self, e: String) {
        if self.best.is_none() {
            self.best = Some(e);
        }
    }

    /// Whether `ca` may issue the next certificate down `path`.
    fn usable_ca(&self, ca: &Cert, path: &[&Cert]) -> Result<(), String> {
        let name = ca.display_name();
        if !ca.is_ca {
            return Err(format!("`{name}` issued a certificate but isn't a certificate authority"));
        }
        if let Some(ku) = ca.key_usage
            && ku & KEY_CERT_SIGN == 0
        {
            return Err(format!("`{name}` isn't allowed to sign certificates (key usage)"));
        }
        if let Some(eku) = &ca.eku
            && !eku.iter().any(|o| o == OID_SERVER_AUTH || o == OID_ANY_EKU)
        {
            return Err(format!("`{name}` isn't allowed to issue server certificates (extended key usage)"));
        }
        check_dates(ca, self.now)?;
        // Intermediates below this one, not counting self-issued ones (§4.2.1.9).
        let below = path[1..].iter().filter(|p| !p.self_issued()).count() as u64;
        if let Some(limit) = ca.path_len
            && below > limit
        {
            return Err(format!("`{name}` allows {limit} intermediate certificate(s) below it, and the chain has more"));
        }
        Ok(())
    }

    /// Name constraints of every CA on the path (and the root) applied to
    /// the leaf's names (§4.2.1.10, DNS and IP forms).
    fn constraints_hold(&mut self, path: &[&Cert], root: Option<&Cert>) -> bool {
        let leaf = path[0];
        for ca in path[1..].iter().copied().chain(root) {
            for name in &leaf.dns_names {
                if let Err(e) = name_allowed(ca, &Name::Dns(name)) {
                    self.note(e);
                    return false;
                }
            }
            for ip in &leaf.ip_addrs {
                if let Err(e) = name_allowed(ca, &Name::Ip(ip)) {
                    self.note(e);
                    return false;
                }
            }
        }
        true
    }
}

enum Name<'a> {
    Dns(&'a str),
    Ip(&'a [u8]),
}

fn dns_within(name: &str, base: &str) -> bool {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    if base.is_empty() {
        return true;
    }
    match base.strip_prefix('.') {
        Some(_) => name.ends_with(base),
        None => name == base || name.ends_with(&format!(".{base}")),
    }
}

fn subtree_matches(t: &Subtree, n: &Name) -> Option<bool> {
    match (t, n) {
        (Subtree::Dns(base), Name::Dns(name)) => Some(dns_within(name, base)),
        (Subtree::Ip(addr, mask), Name::Ip(ip)) if addr.len() == ip.len() => {
            Some(ip.iter().zip(addr).zip(mask).all(|((i, a), m)| i & m == a & m))
        }
        (Subtree::Ip(..), Name::Ip(_)) => Some(false),
        _ => None, // a different form: doesn't apply
    }
}

fn name_allowed(ca: &Cert, n: &Name) -> Result<(), String> {
    let shown = match n {
        Name::Dns(d) => d.to_string(),
        Name::Ip(ip) => ip_text(ip),
    };
    if ca.excluded.iter().any(|t| subtree_matches(t, n) == Some(true)) {
        return Err(format!("`{}` may not issue certificates for `{shown}` (name constraints)", ca.display_name()));
    }
    let applicable: Vec<bool> = ca.permitted.iter().filter_map(|t| subtree_matches(t, n)).collect();
    if !applicable.is_empty() && !applicable.contains(&true) {
        return Err(format!("`{}` may not issue certificates for `{shown}` (name constraints)", ca.display_name()));
    }
    Ok(())
}

fn ip_text(ip: &[u8]) -> String {
    match ip.len() {
        4 => std::net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]).to_string(),
        16 => std::net::Ipv6Addr::from(<[u8; 16]>::try_from(ip).unwrap()).to_string(),
        _ => format!("{ip:?}"),
    }
}

/// The host against the leaf's subjectAltName (RFC 6125 §6.4): DNS names
/// case-insensitively, `*` only as the whole leftmost label, and IP
/// addresses against IP entries. The common name isn't consulted.
pub fn check_host(leaf: &Cert, host: &str) -> Result<(), String> {
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let ok = match host.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(a)) => leaf.ip_addrs.iter().any(|ip| ip[..] == a.octets()),
        Ok(std::net::IpAddr::V6(a)) => leaf.ip_addrs.iter().any(|ip| ip[..] == a.octets()),
        Err(_) => {
            let host = host.trim_end_matches('.').to_ascii_lowercase();
            leaf.dns_names.iter().any(|san| dns_matches(&san.to_ascii_lowercase(), &host))
        }
    };
    if ok {
        return Ok(());
    }
    let mut covers: Vec<String> = leaf.dns_names.clone();
    covers.extend(leaf.ip_addrs.iter().map(|ip| ip_text(ip)));
    let covers = if covers.is_empty() {
        "no host names".to_string()
    } else if covers.len() > 4 {
        format!("{} and {} more", covers[..4].join(", "), covers.len() - 4)
    } else {
        covers.join(", ")
    };
    Err(format!("the server's certificate isn't valid for `{host}`; it covers {covers}"))
}

fn dns_matches(san: &str, host: &str) -> bool {
    let san = san.trim_end_matches('.');
    if let Some(rest) = san.strip_prefix("*.") {
        // At least two labels after the wildcard, and exactly one label for it.
        if !rest.contains('.') {
            return false;
        }
        return match host.split_once('.') {
            Some((label, tail)) => !label.is_empty() && tail == rest,
            None => false,
        };
    }
    !san.contains('*') && san == host
}
