//! Reading and writing TLS's length-prefixed encodings (RFC 8446 §3).

use super::TlsError;

/// A cursor over received bytes; running past the end is a `decode_error`.
pub struct Reader<'a> {
    data: &'a [u8],
    what: &'static str,
}

impl<'a> Reader<'a> {
    /// `what` names the message in errors.
    pub fn new(data: &'a [u8], what: &'static str) -> Self {
        Reader { data, what }
    }

    fn short(&self) -> TlsError {
        TlsError::protocol(format!("the server sent a malformed {} (decode_error)", self.what))
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], TlsError> {
        if n > self.data.len() {
            return Err(self.short());
        }
        let (head, rest) = self.data.split_at(n);
        self.data = rest;
        Ok(head)
    }

    pub fn u8(&mut self) -> Result<u8, TlsError> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16, TlsError> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    pub fn u24(&mut self) -> Result<usize, TlsError> {
        let b = self.take(3)?;
        Ok((b[0] as usize) << 16 | (b[1] as usize) << 8 | b[2] as usize)
    }

    /// A vector with a one-, two- or three-byte length prefix.
    pub fn vec8(&mut self) -> Result<&'a [u8], TlsError> {
        let n = self.u8()? as usize;
        self.take(n)
    }

    pub fn vec16(&mut self) -> Result<&'a [u8], TlsError> {
        let n = self.u16()? as usize;
        self.take(n)
    }

    pub fn vec24(&mut self) -> Result<&'a [u8], TlsError> {
        let n = self.u24()?;
        self.take(n)
    }

    /// Fails unless every byte was consumed.
    pub fn finish(&self) -> Result<(), TlsError> {
        if self.data.is_empty() { Ok(()) } else { Err(self.short()) }
    }
}

/// Appends `body` with a big-endian length prefix of `width` bytes.
pub fn put_vec(out: &mut Vec<u8>, width: usize, body: &[u8]) {
    let n = body.len();
    assert!(width == 4 || n < 1 << (8 * width), "TLS vector too long");
    out.extend_from_slice(&(n as u32).to_be_bytes()[4 - width..]);
    out.extend_from_slice(body);
}

/// A handshake message: type, 24-bit length, body (RFC 8446 §4).
pub fn handshake(msg_type: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![msg_type];
    put_vec(&mut out, 3, body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_writes_vectors() {
        let mut out = Vec::new();
        put_vec(&mut out, 1, b"ab");
        put_vec(&mut out, 2, b"cde");
        put_vec(&mut out, 3, b"");
        assert_eq!(out, b"\x02ab\x00\x03cde\x00\x00\x00");
        let mut r = Reader::new(&out, "test");
        assert_eq!(r.vec8().unwrap(), b"ab");
        assert_eq!(r.vec16().unwrap(), b"cde");
        assert_eq!(r.vec24().unwrap(), b"");
        r.finish().unwrap();
        assert!(r.u8().is_err());
        let mut r = Reader::new(b"\x05abc", "ServerHello");
        assert_eq!(r.vec8().unwrap_err().message, "the server sent a malformed ServerHello (decode_error)");
    }
}
