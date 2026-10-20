//! Certificate tests against the test PKI in `testdata/` (see
//! `testdata/gencerts.py`), the system root store, and live handshakes.

use super::verify::{WebVerifier, verify_chain};
use super::*;
use crate::net::crypto::hex;
use std::collections::HashMap;
use std::io::{Read, Write};

fn pki() -> HashMap<&'static str, Vec<u8>> {
    include_str!("testdata/chain.txt")
        .lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| {
            let (k, v) = l.split_once(' ').unwrap();
            (k, hex(v))
        })
        .collect()
}

/// 2027-01-01, inside every test certificate's validity unless it says otherwise.
const NOW: i64 = 1798761600;

fn check(names: &[&str], host: &str) -> Result<Cert, String> {
    let p = pki();
    let roots = vec![Cert::parse(&p["root"]).unwrap()];
    let chain: Vec<Vec<u8>> = names.iter().map(|n| p[n].clone()).collect();
    verify_chain(&chain, &roots, host, NOW)
}

fn err(names: &[&str], host: &str) -> String {
    check(names, host).err().expect("should fail")
}

#[test]
fn parses_the_test_pki() {
    let p = pki();
    let leaf = Cert::parse(&p["leaf"]).unwrap();
    assert_eq!(leaf.display_name(), "example.test");
    assert_eq!(leaf.issuer_name(), "AIL Test Intermediate");
    assert_eq!(leaf.dns_names, vec!["example.test", "*.wild.test"]);
    assert_eq!(leaf.ip_addrs.len(), 2);
    assert!(!leaf.is_ca);
    assert!(matches!(leaf.key, PublicKey::Ec(Curve::P256, _)));
    assert_eq!(der::date(leaf.not_before), "2026-01-01");
    let inter = Cert::parse(&p["inter"]).unwrap();
    assert!(inter.is_ca && inter.path_len == Some(0));
    assert!(matches!(inter.key, PublicKey::Rsa(_)));
    let root = Cert::parse(&p["root"]).unwrap();
    assert!(root.self_issued() && matches!(root.key, PublicKey::Ec(Curve::P384, _)));
    assert!(Cert::parse(&p["unknown_critical"]).unwrap_err().contains("unknown critical extension"));
}

#[test]
fn accepts_good_chains_and_names() {
    for host in ["example.test", "EXAMPLE.test.", "a.wild.test", "127.0.0.1", "::1", "[::1]"] {
        check(&["leaf", "inter"], host).unwrap_or_else(|e| panic!("{host}: {e}"));
    }
    // RSA leaf; a PSS-signed leaf; SHA-512; the root sent too; order and
    // extras don't matter.
    check(&["rsa_leaf", "inter"], "rsa.test").unwrap();
    check(&["pss_signed", "inter"], "pss.test").unwrap();
    check(&["sha512_leaf", "inter"], "example.test").unwrap();
    check(&["leaf", "inter", "root"], "example.test").unwrap();
    check(&["leaf", "self_signed", "constrained", "inter"], "example.test").unwrap();
    check(&["constrained_ok", "constrained"], "x.allowed.test").unwrap();
}

#[test]
fn refuses_wrong_names() {
    for host in ["other.test", "a.b.wild.test", "wild.test", "example.test.evil", "127.0.0.2"] {
        let e = err(&["leaf", "inter"], host);
        assert!(e.contains("isn't valid for"), "{host}: {e}");
    }
    assert_eq!(
        err(&["leaf", "inter"], "other.test"),
        "the server's certificate isn't valid for `other.test`; it covers example.test, *.wild.test, 127.0.0.1, ::1"
    );
}

#[test]
fn refuses_bad_chains_with_reasons() {
    assert_eq!(err(&["expired", "inter"], "example.test"), "the certificate for `example.test` expired on 2026-06-01");
    assert_eq!(
        err(&["future", "inter"], "example.test"),
        "the certificate for `example.test` isn't valid until 2030-01-01"
    );
    assert!(err(&["client_eku", "inter"], "example.test").contains("isn't for servers"));
    assert!(err(&["unknown_critical", "inter"], "example.test").contains("unknown critical extension"));
    assert!(err(&["self_signed"], "example.test").contains("doesn't lead to a trusted root"));
    assert_eq!(
        err(&["leaf"], "example.test"),
        "the certificate for `example.test` doesn't lead to a trusted root: it was issued by `AIL Test Intermediate`, which isn't in the system's root store"
    );
    assert!(err(&["under_not_ca", "not_ca"], "example.test").contains("isn't a certificate authority"));
    assert!(err(&["under_sub_ca", "sub_ca", "inter"], "example.test").contains("intermediate certificate(s) below it"));
    assert!(err(&["constrained_bad", "constrained"], "x.other.test").contains("name constraints"));
    // A tampered signature.
    let p = pki();
    let mut bad = p["leaf"].clone();
    let n = bad.len();
    bad[n - 10] ^= 1;
    let roots = vec![Cert::parse(&p["root"]).unwrap()];
    let e = verify_chain(&[bad, p["inter"].clone()], &roots, "example.test", NOW).unwrap_err();
    assert!(e.contains("doesn't verify"), "{e}");
    // No roots at all.
    assert!(verify_chain(&[p["leaf"].clone(), p["inter"].clone()], &[], "example.test", NOW).is_err());
}

#[test]
fn pem_and_base64() {
    let text = include_str!("testdata/inter.pem");
    assert_eq!(pem_certs(text), vec![pki()["inter"].clone()]);
    assert_eq!(base64("aGVsbG8=").unwrap(), b"hello");
    assert_eq!(base64("aGVsbG8gd29ybGQ").unwrap(), b"hello world");
}

// The platform's root store loads, and nearly every root parses.
#[test]
fn system_roots_load() {
    match roots::system() {
        Ok(r) => {
            assert!(r.certs.len() > 50, "only {} roots from {}", r.certs.len(), r.source);
            eprintln!("note: {} roots from {}", r.certs.len(), r.source);
        }
        Err(e) => eprintln!("note: no system roots here ({e}); skipping"),
    }
}

// The parsers never panic on mutated certificates (ADR 0269, ADR 0368).
#[test]
fn fuzz_certificate_parser() {
    let n: u64 = std::env::var("AIL_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(5000);
    let p = pki();
    let seeds: Vec<&Vec<u8>> = p.values().collect();
    let roots = vec![Cert::parse(&p["root"]).unwrap()];
    let mut state = 0x5eed_u64;
    let mut next = move || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    for _ in 0..n {
        let mut c = seeds[(next() % seeds.len() as u64) as usize].clone();
        for _ in 0..1 + next() % 4 {
            let i = (next() % c.len() as u64) as usize;
            match next() % 4 {
                0 => c[i] ^= 1 << (next() % 8),
                1 => c[i] = next() as u8,
                2 => c.insert(i, next() as u8),
                _ => c.truncate(i.max(1)),
            }
        }
        if let Ok(cert) = Cert::parse(&c) {
            let _ = cert.signed_by(&roots[0].key);
            let _ = verify_chain(&[c.clone(), p["inter"].clone()], &roots, "example.test", NOW);
        }
    }
}

/// Full handshakes with real certificate checks: `openssl s_server`
/// serving the test chain for an ECDSA P-256 leaf (CertificateVerify
/// ecdsa_secp256r1_sha256) and an RSA leaf (rsa_pss_rsae_sha256), the
/// test root as the only trusted root. Skips without OpenSSL 1.1.1+.
#[test]
fn live_handshake_with_certificate_checks() {
    use crate::net::tls::client::tests::openssl::Server;
    use crate::net::tls::{Config, connect};
    let p = pki();
    let roots = vec![Cert::parse(&p["root"]).unwrap()];
    let inter_pem = include_str!("testdata/inter.pem");
    for (key, leaf, host) in [
        (include_str!("testdata/localhost_leaf.key.pem"), include_str!("testdata/localhost_leaf.pem"), "localhost"),
        (include_str!("testdata/rsa_leaf.key.pem"), include_str!("testdata/rsa_leaf.pem"), "localhost"),
    ] {
        let Some(server) = Server::from_pem(key, leaf, inter_pem) else {
            eprintln!("note: no OpenSSL 1.1.1+ `openssl`; skipping the live certificate test");
            return;
        };
        let mut running = server.start(&["-tls1_3", "-cert_chain", "chain.pem"]);
        let verifier = WebVerifier { roots: &roots, now: None };
        let config = Config { host, verifier: &verifier };
        let mut tls = connect(running.connect(), &config).unwrap_or_else(|e| panic!("{host}: {e}"));
        tls.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        let mut body = Vec::new();
        tls.read_to_end(&mut body).unwrap();
        assert!(String::from_utf8_lossy(&body).starts_with("HTTP/1.0 200 ok"));
        // The wrong name fails with a certificate error naming the cause.
        let mut running = server.start(&["-tls1_3", "-cert_chain", "chain.pem"]);
        let config = Config { host: "wrong.test", verifier: &verifier };
        let e = connect(running.connect(), &config).err().unwrap();
        assert_eq!(e.kind, crate::net::tls::Kind::Certificate);
        assert!(e.message.contains("isn't valid for `wrong.test`"), "{}", e.message);
    }
}

/// Real servers on the public internet, opt in: `AIL_TLS_PUBLIC=1 ./ail
/// selftest public_sites -- --nocapture`. A table of suite and result per
/// site; it fails if any site fails. Not part of the gate, which must not
/// need the network.
#[test]
fn public_sites() {
    use crate::net::tls::{Config, connect};
    if std::env::var("AIL_TLS_PUBLIC").as_deref() != Ok("1") {
        return;
    }
    let roots = roots::system().expect("system roots");
    let sites = [
        "example.com", "www.google.com", "github.com", "api.github.com", "www.cloudflare.com",
        "letsencrypt.org", "en.wikipedia.org", "www.apple.com", "www.microsoft.com", "pypi.org",
        "crates.io", "www.amazon.com", "news.ycombinator.com", "www.rust-lang.org",
    ];
    let mut failed = Vec::new();
    for host in sites {
        let result = (|| -> Result<String, String> {
            let addr = std::net::ToSocketAddrs::to_socket_addrs(&(host, 443)).map_err(|e| e.to_string())?.next().ok_or("no address")?;
            let s = std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(10)).map_err(|e| e.to_string())?;
            s.set_read_timeout(Some(std::time::Duration::from_secs(20))).unwrap();
            let verifier = WebVerifier { roots: &roots.certs, now: None };
            let mut tls = connect(s, &Config { host, verifier: &verifier }).map_err(|e| e.message)?;
            write!(tls, "HEAD / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\nUser-Agent: ail\r\n\r\n").map_err(|e| e.to_string())?;
            let mut head = [0u8; 12];
            tls.read_exact(&mut head).map_err(|e| e.to_string())?;
            Ok(format!("{} {}", tls.suite(), String::from_utf8_lossy(&head)))
        })();
        match result {
            Ok(r) => eprintln!("ok    {host:24} {r}"),
            Err(e) => {
                eprintln!("FAIL  {host:24} {e}");
                failed.push(host);
            }
        }
    }
    assert!(failed.is_empty(), "failed: {failed:?}");
}
