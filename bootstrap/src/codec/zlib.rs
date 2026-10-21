//! zlib streams (RFC 1950): a two-byte header, DEFLATE, and an Adler-32
//! of the output. PDF Flate streams, PNG and HTTP `deflate` use this
//! wrapper (std.compress.7, ADR 0443).

use super::deflate::deflate;
use super::inflate::inflate_prefix;
use super::CodecError;

pub fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in data.chunks(5552) {
        for &x in chunk {
            a += x as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

pub fn zlib(data: &[u8], level: u32) -> Vec<u8> {
    let mut out = vec![0x78, 0x9c];
    out.extend(deflate(data, level));
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

pub fn unzlib(data: &[u8], limit: usize) -> Result<Vec<u8>, CodecError> {
    // std.compress.10: the zlib corrupt, checksum and unsupported messages.
    let bad = |m: &str| CodecError::new("corrupt", m);
    if data.len() < 6 {
        return Err(bad("not zlib data"));
    }
    let (cmf, flg) = (data[0], data[1]);
    if cmf & 0x0f != 8 || cmf >> 4 > 7 || (((cmf as u32) << 8) | flg as u32) % 31 != 0 {
        return Err(bad("not zlib data"));
    }
    if flg & 0x20 != 0 {
        return Err(CodecError::new("unsupported", "zlib preset dictionaries are not supported"));
    }
    let (out, used) = inflate_prefix(&data[2..], limit)?;
    let t = data.get(2 + used..2 + used + 4).ok_or_else(|| bad("truncated zlib trailer"))?;
    if u32::from_be_bytes([t[0], t[1], t[2], t[3]]) != adler32(&out) {
        return Err(CodecError::new("checksum", "zlib Adler-32 mismatch"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_answers() {
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
        assert_eq!(adler32(b""), 1);
        // `zlib.compress(b"hello")` from Python's zlib at its default level.
        let z = [0x78, 0x9c, 0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x07, 0x00, 0x06, 0x2c, 0x02, 0x15];
        assert_eq!(unzlib(&z, 1 << 20).unwrap(), b"hello");
    }

    #[test]
    fn round_trip_and_damage() {
        let data: Vec<u8> = (0..20000u32).map(|i| (i % 97) as u8).collect();
        let z = zlib(&data, 6);
        assert_eq!(unzlib(&z, 1 << 20).unwrap(), data);
        let mut bad = z.clone();
        let n = bad.len();
        bad[n - 1] ^= 1;
        assert_eq!(unzlib(&bad, 1 << 20).unwrap_err().kind, "checksum");
        assert!(unzlib(b"nope nope", 1 << 20).is_err());
    }
}
