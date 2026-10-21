//! TLS causes: std.tls.1 (handshake and records) and std.tls.3 (protocol and roots).
//! An in-house TLS 1.3 client (RFC 8446; ADR 0360, ADR 0367). Client side
//! only: X25519 key share; TLS_CHACHA20_POLY1305_SHA256,
//! TLS_AES_128_GCM_SHA256 and TLS_AES_256_GCM_SHA384; no client
//! certificates, no 0-RTT, no session resumption, no TLS 1.2.
//!
//! Certificate checking is a `Verifier` supplied by the caller; the real
//! one (X.509 chains against the platform roots) is slice 3 (ADR 0368).

#![allow(dead_code)]

pub(crate) mod client;
mod codec;
mod record;
mod schedule;

pub use client::{Config, connect};

/// What went wrong, for `std.net`'s `tls` failure (std.net.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// The peer broke the protocol, sent an alert, or chose something this
    /// client doesn't support.
    Protocol,
    /// The certificate or its signature didn't check out.
    Certificate,
    /// The transport failed (connection reset, timeout, end of stream).
    Io(std::io::ErrorKind),
}

/// A TLS failure: its kind and a sentence naming the cause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsError {
    pub kind: Kind,
    pub message: String,
}

impl TlsError {
    pub fn protocol(message: impl Into<String>) -> Self {
        TlsError { kind: Kind::Protocol, message: message.into() }
    }

    pub fn certificate(message: impl Into<String>) -> Self {
        TlsError { kind: Kind::Certificate, message: message.into() }
    }
}

impl From<std::io::Error> for TlsError {
    fn from(e: std::io::Error) -> Self {
        let message = if e.kind() == std::io::ErrorKind::UnexpectedEof {
            "the server closed the connection during the TLS exchange".to_string()
        } else {
            e.to_string()
        };
        TlsError { kind: Kind::Io(e.kind()), message }
    }
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Checks the server's identity (RFC 8446 §4.4.2–4.4.3): that `chain` (leaf
/// first, DER) is trusted for `host`, and that `signature` under the leaf's
/// key and `scheme` covers `signed`.
pub trait Verifier {
    fn verify(
        &self,
        host: &str,
        chain: &[Vec<u8>],
        scheme: u16,
        signed: &[u8],
        signature: &[u8],
    ) -> Result<(), TlsError>;
}

/// Signature schemes offered in `signature_algorithms` (RFC 8446 §4.2.3),
/// most preferred first: those the certificate verifier implements.
pub const SIGNATURE_SCHEMES: [u16; 8] = [
    0x0403, // ecdsa_secp256r1_sha256
    0x0503, // ecdsa_secp384r1_sha384
    0x0804, // rsa_pss_rsae_sha256
    0x0805, // rsa_pss_rsae_sha384
    0x0806, // rsa_pss_rsae_sha512
    0x0401, // rsa_pkcs1_sha256 (certificates only)
    0x0501, // rsa_pkcs1_sha384 (certificates only)
    0x0601, // rsa_pkcs1_sha512 (certificates only)
];

/// An alert's description (RFC 8446 §6), for messages.
fn alert_name(code: u8) -> &'static str {
    match code {
        0 => "close_notify",
        10 => "unexpected_message",
        20 => "bad_record_mac",
        22 => "record_overflow",
        40 => "handshake_failure",
        42 => "bad_certificate",
        43 => "unsupported_certificate",
        44 => "certificate_revoked",
        45 => "certificate_expired",
        46 => "certificate_unknown",
        47 => "illegal_parameter",
        48 => "unknown_ca",
        49 => "access_denied",
        50 => "decode_error",
        51 => "decrypt_error",
        70 => "protocol_version",
        71 => "insufficient_security",
        80 => "internal_error",
        86 => "inappropriate_fallback",
        90 => "user_canceled",
        109 => "missing_extension",
        110 => "unsupported_extension",
        112 => "unrecognized_name",
        113 => "bad_certificate_status_response",
        115 => "unknown_psk_identity",
        116 => "certificate_required",
        120 => "no_application_protocol",
        _ => "unknown alert",
    }
}
