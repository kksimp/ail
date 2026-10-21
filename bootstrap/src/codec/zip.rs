//! Zip archives (PKWARE APPNOTE 6.3): store and deflate, no encryption, no
//! zip64 (std.compress.5). Writing is deterministic: entries sorted by
//! name, a fixed 1980-01-01 timestamp. Reading checks every CRC-32 and
//! refuses names that would escape the target directory.

use super::crc32::crc32;
use super::deflate::deflate;
use super::inflate::inflate;
use super::CodecError;

/// One file in an archive: its `/`-separated relative name and contents.
#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub data: Vec<u8>,
}

fn le16(v: usize) -> [u8; 2] {
    (v as u16).to_le_bytes()
}
fn le32(v: usize) -> [u8; 4] {
    (v as u32).to_le_bytes()
}

pub fn write(entries: &mut [Entry], level: u32) -> Result<Vec<u8>, CodecError> {
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    if entries.len() > 0xffff {
        return Err(CodecError::new("too_big", "more than 65535 files; zip64 is not supported"));
    }
    let (mut out, mut central) = (Vec::new(), Vec::new());
    for e in entries.iter() {
        if e.data.len() >= 0xffff_ffff || out.len() >= 0xffff_ffff {
            return Err(CodecError::new("too_big", "over 4 GiB; zip64 is not supported"));
        }
        let crc = crc32(&e.data);
        let packed = deflate(&e.data, level);
        let (method, body) = if packed.len() < e.data.len() { (8u16, packed) } else { (0u16, e.data.clone()) };
        let name = e.name.as_bytes();
        let offset = out.len();
        // 0x0800: names are UTF-8. Time 00:00:00, date 1980-01-01 (0x0021).
        let fixed = [&[20u8, 0][..], &[0, 8], &method.to_le_bytes(), &[0, 0], &[0x21, 0]].concat();
        out.extend_from_slice(b"PK\x03\x04");
        out.extend_from_slice(&fixed);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&le32(body.len()));
        out.extend_from_slice(&le32(e.data.len()));
        out.extend_from_slice(&le16(name.len()));
        out.extend_from_slice(&[0, 0]);
        out.extend_from_slice(name);
        out.extend_from_slice(&body);
        central.extend_from_slice(b"PK\x01\x02");
        central.extend_from_slice(&[20, 3, 20, 0]);
        central.extend_from_slice(&fixed[2..]);
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&le32(body.len()));
        central.extend_from_slice(&le32(e.data.len()));
        central.extend_from_slice(&le16(name.len()));
        central.extend_from_slice(&[0; 8]);
        // External attributes: a regular file, mode 644, for Unix readers.
        let mode: u32 = 0o100644 << 16;
        central.extend_from_slice(&mode.to_le_bytes());
        central.extend_from_slice(&le32(offset));
        central.extend_from_slice(name);
    }
    let start = out.len();
    out.extend_from_slice(&central);
    out.extend_from_slice(b"PK\x05\x06\0\0\0\0");
    out.extend_from_slice(&le16(entries.len()));
    out.extend_from_slice(&le16(entries.len()));
    out.extend_from_slice(&le32(central.len()));
    out.extend_from_slice(&le32(start));
    out.extend_from_slice(&[0, 0]);
    Ok(out)
}

fn bad(msg: &str) -> CodecError {
    CodecError::new("corrupt", msg)
}

fn u16_at(d: &[u8], p: usize) -> Result<usize, CodecError> {
    d.get(p..p + 2).map(|b| u16::from_le_bytes([b[0], b[1]]) as usize).ok_or_else(|| bad("truncated zip data"))
}
fn u32_at(d: &[u8], p: usize) -> Result<usize, CodecError> {
    d.get(p..p + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize).ok_or_else(|| bad("truncated zip data"))
}

/// std.compress.5: a name is safe when it is relative and has no `..`
/// or empty component (and no drive letter or backslash).
pub fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.contains('\0')
        && name.trim_end_matches('/').split('/').all(|c| !c.is_empty() && c != "." && c != "..")
}

/// Read every file of an archive; directory entries (names ending in `/`)
/// come back with empty data and the trailing slash kept. `limit` caps the
/// total uncompressed size.
pub fn read(d: &[u8], limit: usize) -> Result<Vec<Entry>, CodecError> {
    let eocd = (0..d.len().saturating_sub(21))
        .rev()
        .take(65535 + 22)
        .find(|&p| &d[p..p + 4] == b"PK\x05\x06")
        .ok_or_else(|| bad("not a zip archive"))?;
    let count = u16_at(d, eocd + 10)?;
    let mut p = u32_at(d, eocd + 16)?;
    let mut out = Vec::new();
    let mut total = 0usize;
    for _ in 0..count {
        if d.get(p..p + 4) != Some(b"PK\x01\x02") {
            return Err(bad("bad zip central directory"));
        }
        let flags = u16_at(d, p + 8)?;
        let method = u16_at(d, p + 10)?;
        let crc = u32_at(d, p + 16)?;
        let csize = u32_at(d, p + 20)?;
        let usize_ = u32_at(d, p + 24)?;
        let (nlen, xlen, clen) = (u16_at(d, p + 28)?, u16_at(d, p + 30)?, u16_at(d, p + 32)?);
        let local = u32_at(d, p + 42)?;
        let name = std::str::from_utf8(d.get(p + 46..p + 46 + nlen).ok_or_else(|| bad("truncated zip data"))?)
            .map_err(|_| bad("zip name is not UTF-8"))?
            .to_string();
        p += 46 + nlen + xlen + clen;
        if flags & 1 != 0 {
            return Err(CodecError::new("unsupported", "encrypted zip entries are not supported"));
        }
        if !safe_name(&name) {
            return Err(CodecError::new("unsafe_path", format!("unsafe name in archive: {name}")));
        }
        if d.get(local..local + 4) != Some(b"PK\x03\x04") {
            return Err(bad("bad zip local header"));
        }
        let start = local + 30 + u16_at(d, local + 26)? + u16_at(d, local + 28)?;
        let raw = d.get(start..start + csize).ok_or_else(|| bad("truncated zip data"))?;
        total = total.saturating_add(usize_);
        if total > limit {
            return Err(CodecError::new("too_big", "archive expands past the size limit"));
        }
        let data = if name.ends_with('/') {
            Vec::new()
        } else {
            match method {
                0 => raw.to_vec(),
                8 => inflate(raw, usize_)?,
                m => return Err(CodecError::new("unsupported", format!("zip method {m} is not supported"))),
            }
        };
        if data.len() != usize_ || crc32(&data) as usize != crc {
            return Err(CodecError::new("checksum", format!("zip CRC-32 or size mismatch in {name}")));
        }
        out.push(Entry { name, data });
    }
    Ok(out)
}
