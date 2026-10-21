//! gzip (RFC 1952) over `deflate`/`inflate`. The header written is the
//! plain ten bytes with no timestamp, so output is reproducible
//! (std.compress.3). Reading accepts the optional header fields and
//! several members in a row, and checks the CRC-32 and size of each.

use super::crc32::crc32;
use super::deflate::deflate;
use super::inflate::inflate_prefix;
use super::CodecError;

pub fn gzip(data: &[u8], level: u32) -> Vec<u8> {
    let mut out = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, if level >= 9 { 2 } else if level <= 1 { 4 } else { 0 }, 3];
    out.extend(deflate(data, level));
    out.extend_from_slice(&crc32(data).to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out
}

// std.compress.10: the gzip corrupt and checksum messages.
fn bad(msg: &str) -> CodecError {
    CodecError::new("corrupt", msg)
}

pub fn gunzip(data: &[u8], limit: usize) -> Result<Vec<u8>, CodecError> {
    let mut out = Vec::new();
    let mut at = 0;
    loop {
        let d = &data[at..];
        // std.compress.8: no gzip magic `1f 8b` (or too short for a header) is corrupt.
        if d.len() < 18 || d[0] != 0x1f || d[1] != 0x8b {
            return Err(bad("not gzip data"));
        }
        if d[2] != 8 {
            return Err(bad("unknown gzip compression method"));
        }
        let flags = d[3];
        if flags & 0xe0 != 0 {
            return Err(bad("reserved gzip flags set"));
        }
        let mut p = 10;
        if flags & 4 != 0 {
            let n = *d.get(p).ok_or_else(|| bad("truncated gzip header"))? as usize
                | (*d.get(p + 1).ok_or_else(|| bad("truncated gzip header"))? as usize) << 8;
            p += 2 + n;
        }
        for bit in [8u8, 16] {
            if flags & bit != 0 {
                let z = d.get(p..).and_then(|s| s.iter().position(|&b| b == 0)).ok_or_else(|| bad("truncated gzip header"))?;
                p += z + 1;
            }
        }
        if flags & 2 != 0 {
            p += 2;
        }
        let body = d.get(p..).ok_or_else(|| bad("truncated gzip header"))?;
        let (chunk, used) = inflate_prefix(body, limit.saturating_sub(out.len()))?;
        let trailer = body.get(used..used + 8).ok_or_else(|| bad("truncated gzip trailer"))?;
        let crc = u32::from_le_bytes(trailer[..4].try_into().unwrap());
        let size = u32::from_le_bytes(trailer[4..].try_into().unwrap());
        if crc != crc32(&chunk) {
            return Err(CodecError::new("checksum", "gzip CRC-32 mismatch"));
        }
        if size != chunk.len() as u32 {
            return Err(CodecError::new("checksum", "gzip size mismatch"));
        }
        out.extend(chunk);
        at += p + used + 8;
        if at >= data.len() {
            return Ok(out);
        }
    }
}
