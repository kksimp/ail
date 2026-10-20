//! `std.hash` and `std.compress` at run time (std.hash.1-4,
//! std.compress.1-6, ADRs 0442-0446): thin wrappers over `crate::codec` and
//! `net::crypto`. Binary data is a `List[int]` of bytes until the `bytes`
//! type lands (ADR 0442).

use std::rc::Rc;

use super::stdlib::fail;
use super::{Interp, Value};
use crate::codec::{self, CodecError, gzip, hash64, zip};
use crate::diag::Pos;
use crate::net::crypto::{hmac::Hmac, sha2::{Hash, Sha256, Sha384, Sha512}};

/// std.compress.4: output past this many bytes is refused, the size of the
/// longest `str` (ADR 0271).
const LIMIT: usize = 1 << 30;
/// std.compress.3: the level every call uses.
const LEVEL: u32 = 6;

fn bytes_of(v: &Value) -> Vec<u8> {
    match v {
        Value::Str(s) => s.as_bytes().to_vec(),
        Value::List(xs) => xs.iter().map(|x| if let Value::Int(i) = x { *i as u8 } else { 0 }).collect(),
        other => unreachable!("checked: str or List[int], found {other:?}"),
    }
}

fn text(v: &Value) -> &str {
    match v {
        Value::Str(s) => s,
        other => unreachable!("checked: str, found {other:?}"),
    }
}

fn list(b: Vec<u8>) -> Value {
    Value::List(Rc::new(b.into_iter().map(|x| Value::Int(x as i64)).collect()))
}

impl<'a, W: std::io::Write> Interp<'a, W> {
    pub(super) fn codec_call(&mut self, module: &str, f: &str, v: &[Value], pos: Pos) -> Value {
        let s = |x: String| Value::Str(x.into());
        match (module, f) {
            ("hash", "sha256") => s(codec::hex(&Sha256::digest(&bytes_of(&v[0])))),
            ("hash", "sha384") => s(codec::hex(&Sha384::digest(&bytes_of(&v[0])))),
            ("hash", "sha512") => s(codec::hex(&Sha512::digest(&bytes_of(&v[0])))),
            ("hash", "hmac_sha256") => s(codec::hex(&Hmac::<Sha256>::mac(&bytes_of(&v[0]), &bytes_of(&v[1])))),
            ("hash", "crc32") => Value::Int(codec::crc32::crc32(&bytes_of(&v[0])) as i64),
            ("hash", "hash64") => Value::Int(hash64::hash64(&bytes_of(&v[0])) as i64),
            ("compress", "deflate") => list(codec::deflate::deflate(&bytes_of(&v[0]), LEVEL)),
            ("compress", "gzip") => list(gzip::gzip(&bytes_of(&v[0]), LEVEL)),
            ("compress", "inflate") => self.decoded(codec::inflate::inflate(&bytes_of(&v[0]), LIMIT), pos),
            ("compress", "gunzip") => self.decoded(gzip::gunzip(&bytes_of(&v[0]), LIMIT), pos),
            ("compress", "gzip_file") => {
                let r = read_file(text(&v[0])).map(|d| gzip::gzip(&d, LEVEL)).and_then(|z| write_file(text(&v[1]), &z));
                self.unit_result(r, pos)
            }
            ("compress", "gunzip_file") => {
                let r = read_file(text(&v[0]))
                    .and_then(|d| gzip::gunzip(&d, LIMIT))
                    .and_then(|d| write_file(text(&v[1]), &d));
                self.unit_result(r, pos)
            }
            ("compress", "zip_dir") => self.unit_result(zip_dir(text(&v[0]), text(&v[1])), pos),
            ("compress", "unzip") => self.unit_result(unzip(text(&v[0]), text(&v[1])), pos),
            _ => unreachable!("stdlib::MODULES lists `{module}.{f}`"),
        }
    }

    fn error(&self, e: CodecError, pos: Pos) -> Value {
        let st = self.structs["compress.CompressError"];
        fail(Value::Struct(st, Rc::new(vec![Value::Str(e.kind.into()), Value::Str(e.message.into())])), pos)
    }

    fn decoded(&self, r: Result<Vec<u8>, CodecError>, pos: Pos) -> Value {
        match r.and_then(|b| {
            String::from_utf8(b).map_err(|_| CodecError::new("invalid_utf8", "the data is not valid UTF-8 text"))
        }) {
            Ok(t) => Value::Str(t.into()),
            Err(e) => self.error(e, pos),
        }
    }

    fn unit_result(&self, r: Result<(), CodecError>, pos: Pos) -> Value {
        match r {
            Ok(()) => Value::Unit,
            Err(e) => self.error(e, pos),
        }
    }
}

fn io(path: &str, e: std::io::Error) -> CodecError {
    CodecError::new(crate::stdlib::error_kind(&e), format!("{path}: {e}"))
}

fn read_file(path: &str) -> Result<Vec<u8>, CodecError> {
    std::fs::read(path).map_err(|e| io(path, e))
}

fn write_file(path: &str, data: &[u8]) -> Result<(), CodecError> {
    std::fs::write(path, data).map_err(|e| io(path, e))
}

/// std.compress.5: every file under `dir` (directories that hold nothing
/// as `name/` entries), named relative to `dir` with `/`.
fn zip_dir(dir: &str, out: &str) -> Result<(), CodecError> {
    fn walk(root: &std::path::Path, rel: &str, into: &mut Vec<zip::Entry>) -> Result<(), CodecError> {
        let here = if rel.is_empty() { root.to_path_buf() } else { root.join(rel) };
        let mut any = false;
        let mut names: Vec<_> = std::fs::read_dir(&here)
            .map_err(|e| io(&here.to_string_lossy(), e))?
            .filter_map(|e| e.ok())
            .collect();
        names.sort_by_key(|e| e.file_name());
        for e in names {
            let name = e.file_name().to_string_lossy().into_owned();
            let path = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
            let t = e.file_type().map_err(|e| io(&path, e))?;
            if t.is_symlink() {
                continue;
            }
            any = true;
            if t.is_dir() {
                walk(root, &path, into)?;
            } else {
                let data = std::fs::read(root.join(&path)).map_err(|e| io(&path, e))?;
                into.push(zip::Entry { name: path, data });
            }
        }
        if !any && !rel.is_empty() {
            into.push(zip::Entry { name: format!("{rel}/"), data: Vec::new() });
        }
        Ok(())
    }
    let mut entries = Vec::new();
    walk(std::path::Path::new(dir), "", &mut entries)?;
    let packed = zip::write(&mut entries, LEVEL)?;
    write_file(out, &packed)
}

/// std.compress.5: extract into `dir`, creating it; a name that would leave
/// `dir` fails before anything is written.
fn unzip(archive: &str, dir: &str) -> Result<(), CodecError> {
    let entries = zip::read(&read_file(archive)?, LIMIT)?;
    let root = std::path::Path::new(dir);
    std::fs::create_dir_all(root).map_err(|e| io(dir, e))?;
    for e in entries {
        let target = root.join(e.name.trim_end_matches('/'));
        if e.name.ends_with('/') {
            std::fs::create_dir_all(&target).map_err(|x| io(&e.name, x))?;
        } else {
            if let Some(p) = target.parent() {
                std::fs::create_dir_all(p).map_err(|x| io(&e.name, x))?;
            }
            std::fs::write(&target, &e.data).map_err(|x| io(&e.name, x))?;
        }
    }
    Ok(())
}
