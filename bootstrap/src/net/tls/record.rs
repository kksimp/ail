//! The record layer (RFC 8446 §5): framing, protection, and the limits a
//! peer's records must keep.

use std::io::{Read, Write};

use super::TlsError;
use super::schedule::TrafficKeys;

pub const CHANGE_CIPHER_SPEC: u8 = 20;
pub const ALERT: u8 = 21;
pub const HANDSHAKE: u8 = 22;
pub const APPLICATION_DATA: u8 = 23;

/// The largest plaintext fragment (§5.1).
const MAX_PLAINTEXT: usize = 1 << 14;
/// The largest protected record body: fragment, type byte, padding and tag
/// together may add at most 256 bytes (§5.2).
const MAX_CIPHERTEXT: usize = MAX_PLAINTEXT + 256;

/// Records over a byte stream, with optional protection per direction.
pub struct RecordLayer<S> {
    pub stream: S,
    pub read_keys: Option<TrafficKeys>,
    pub write_keys: Option<TrafficKeys>,
}

impl<S: Read + Write> RecordLayer<S> {
    pub fn new(stream: S) -> Self {
        RecordLayer { stream, read_keys: None, write_keys: None }
    }

    /// Reads one record: its content type and plaintext. A protected record
    /// is decrypted and its padding removed (§5.4); an unprotected
    /// `change_cipher_spec` is passed through for the caller to judge.
    pub fn read(&mut self) -> Result<(u8, Vec<u8>), TlsError> {
        let mut header = [0u8; 5];
        self.stream.read_exact(&mut header)?;
        let (ty, len) = (header[0], u16::from_be_bytes([header[3], header[4]]) as usize);
        if header[1] != 3 {
            return Err(TlsError::protocol(format!(
                "the server didn't answer with TLS (its first bytes were {:02x} {:02x} {:02x}); is this an `http` port?",
                header[0], header[1], header[2]
            )));
        }
        let limit = if self.read_keys.is_some() && ty == APPLICATION_DATA { MAX_CIPHERTEXT } else { MAX_PLAINTEXT };
        if len > limit {
            return Err(TlsError::protocol(format!("the server sent a {len}-byte record (record_overflow)")));
        }
        let mut body = vec![0u8; len];
        self.stream.read_exact(&mut body)?;
        let Some(keys) = self.read_keys.as_mut() else {
            if ty == APPLICATION_DATA {
                return Err(TlsError::protocol("the server sent encrypted data before the keys were agreed"));
            }
            return Ok((ty, body));
        };
        match ty {
            APPLICATION_DATA => {}
            // §5: a change_cipher_spec may arrive unprotected during the
            // handshake for middlebox compatibility; the caller checks when.
            CHANGE_CIPHER_SPEC => return Ok((ty, body)),
            _ => {
                return Err(TlsError::protocol(format!(
                    "the server sent an unprotected record of type {ty} after the keys were agreed (unexpected_message)"
                )));
            }
        }
        let mut inner = keys
            .open(&header, &body)
            .map_err(|_| TlsError::protocol("a record from the server failed authentication (bad_record_mac)"))?;
        // The real content type is the last nonzero byte (§5.4).
        let Some(end) = inner.iter().rposition(|&b| b != 0) else {
            return Err(TlsError::protocol("the server sent a record with no content type (unexpected_message)"));
        };
        let real = inner[end];
        inner.truncate(end);
        if inner.len() > MAX_PLAINTEXT {
            return Err(TlsError::protocol("the server sent an oversized record (record_overflow)"));
        }
        Ok((real, inner))
    }

    /// Writes `data` as one or more records of type `ty`, protected when
    /// write keys are set.
    pub fn write(&mut self, ty: u8, data: &[u8]) -> Result<(), TlsError> {
        let mut out = Vec::with_capacity(data.len() + 32);
        let chunks: Vec<&[u8]> = if data.is_empty() { vec![data] } else { data.chunks(MAX_PLAINTEXT).collect() };
        for chunk in chunks {
            match self.write_keys.as_mut() {
                None => {
                    // The very first ClientHello uses 0x0301 for old
                    // middleboxes (§5.1); everything else uses 0x0303.
                    let version = if ty == HANDSHAKE && chunk.first() == Some(&1) { 0x01 } else { 0x03 };
                    out.extend_from_slice(&[ty, 3, version]);
                    out.extend_from_slice(&(chunk.len() as u16).to_be_bytes());
                    out.extend_from_slice(chunk);
                }
                Some(keys) => {
                    let mut inner = chunk.to_vec();
                    inner.push(ty);
                    let header = [APPLICATION_DATA, 3, 3, 0, 0];
                    let mut header = header;
                    header[3..].copy_from_slice(&((inner.len() + 16) as u16).to_be_bytes());
                    let sealed = keys.seal(&header, &inner);
                    out.extend_from_slice(&header);
                    out.extend_from_slice(&sealed);
                }
            }
        }
        self.stream.write_all(&out)?;
        self.stream.flush()?;
        Ok(())
    }
}
