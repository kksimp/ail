//! The platform's trusted root certificates (ADR 0368), loaded once per
//! process.
//!
//! Order: the file named by `SSL_CERT_FILE` if set (the documented
//! override and fallback on every platform); then on Linux and other
//! Unixes the usual bundle paths; on macOS the system root keychain via
//! `security`, then `/etc/ssl/cert.pem`; on Windows the `ROOT` system
//! store.

use std::sync::OnceLock;

use super::{Cert, pem_certs};

/// Where the roots came from and the parsed certificates.
pub struct Roots {
    pub source: String,
    pub certs: Vec<Cert>,
}

static ROOTS: OnceLock<Result<Roots, String>> = OnceLock::new();

/// The system roots, or why none could be found.
pub fn system() -> Result<&'static Roots, String> {
    ROOTS.get_or_init(load).as_ref().map_err(|e| e.clone())
}

/// Certificates that parse; a root that doesn't (an unusual key or
/// algorithm) is skipped rather than failing every connection.
fn parse_all(ders: Vec<Vec<u8>>) -> Vec<Cert> {
    ders.iter().filter_map(|d| Cert::parse(d).ok()).collect()
}

fn from_file(path: &str) -> Option<Roots> {
    let text = std::fs::read_to_string(path).ok()?;
    let certs = parse_all(pem_certs(&text));
    if certs.is_empty() {
        return None;
    }
    Some(Roots { source: path.to_string(), certs })
}

fn load() -> Result<Roots, String> {
    if let Ok(path) = std::env::var("SSL_CERT_FILE") {
        return from_file(&path).ok_or_else(|| format!("SSL_CERT_FILE names `{path}`, which holds no certificates"));
    }
    platform().ok_or_else(|| {
        "no trusted root certificates found; set SSL_CERT_FILE to a PEM bundle of root certificates".to_string()
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform() -> Option<Roots> {
    // The same list Go and OpenSSL distributions use.
    [
        "/etc/ssl/certs/ca-certificates.crt",                // Debian, Ubuntu, Arch, Alpine
        "/etc/pki/tls/certs/ca-bundle.crt",                  // Fedora, RHEL
        "/etc/ssl/ca-bundle.pem",                            // openSUSE
        "/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem", // CentOS, RHEL 7+
        "/etc/ssl/cert.pem",                                 // Alpine, FreeBSD, OpenBSD
        "/usr/local/share/certs/ca-root-nss.crt",            // FreeBSD
    ]
    .iter()
    .find_map(|p| from_file(p))
}

#[cfg(target_os = "macos")]
fn platform() -> Option<Roots> {
    let keychain = "/System/Library/Keychains/SystemRootCertificates.keychain";
    let exported = std::process::Command::new("/usr/bin/security")
        .args(["find-certificate", "-a", "-p", keychain])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| parse_all(pem_certs(&String::from_utf8_lossy(&o.stdout))))
        .filter(|c| !c.is_empty());
    match exported {
        Some(certs) => Some(Roots { source: format!("the macOS keychain {keychain}"), certs }),
        None => from_file("/etc/ssl/cert.pem"),
    }
}

#[cfg(windows)]
fn platform() -> Option<Roots> {
    use core::ffi::c_void;

    #[repr(C)]
    struct CertContext {
        encoding: u32,
        encoded: *const u8,
        len: u32,
        info: *const c_void,
        store: *const c_void,
    }

    #[link(name = "crypt32")]
    unsafe extern "system" {
        fn CertOpenSystemStoreW(prov: usize, name: *const u16) -> *mut c_void;
        fn CertEnumCertificatesInStore(store: *mut c_void, prev: *const CertContext) -> *const CertContext;
        fn CertCloseStore(store: *mut c_void, flags: u32) -> i32;
    }

    let name: Vec<u16> = "ROOT".encode_utf16().chain([0]).collect();
    let mut ders = Vec::new();
    // SAFETY: the store handle is checked, each context pointer comes from
    // the enumeration and is read before the next call frees it, and the
    // store is closed once.
    unsafe {
        let store = CertOpenSystemStoreW(0, name.as_ptr());
        if store.is_null() {
            return None;
        }
        let mut ctx = CertEnumCertificatesInStore(store, core::ptr::null());
        while !ctx.is_null() {
            let c = &*ctx;
            ders.push(std::slice::from_raw_parts(c.encoded, c.len as usize).to_vec());
            ctx = CertEnumCertificatesInStore(store, ctx);
        }
        CertCloseStore(store, 0);
    }
    let certs = parse_all(ders);
    if certs.is_empty() {
        return None;
    }
    Some(Roots { source: "the Windows ROOT certificate store".to_string(), certs })
}

#[cfg(not(any(unix, windows)))]
fn platform() -> Option<Roots> {
    None
}
