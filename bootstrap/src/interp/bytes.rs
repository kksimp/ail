//! The `bytes` type at run time (spec 03-types, type.bytes.1-13): an
//! immutable sequence of `u8`, shared by copies like a list's storage.

use super::*;

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// type.bytes.11: lowercase hex, two digits a byte.
pub(super) fn hex_encode(b: &[u8]) -> String {
    let mut out = String::with_capacity(b.len() * 2);
    for x in b {
        let _ = write!(out, "{x:02x}");
    }
    out
}

/// type.bytes.13: hex of either case, an even number of digits.
pub(super) fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.as_bytes();
    if s.len() % 2 != 0 {
        return None;
    }
    s.chunks(2).map(|p| Some(((p[0] as char).to_digit(16)? * 16 + (p[1] as char).to_digit(16)?) as u8)).collect()
}

/// type.bytes.12: standard base64 (RFC 4648) with `=` padding.
pub(super) fn base64_encode(b: &[u8]) -> String {
    let mut out = String::with_capacity(b.len().div_ceil(3) * 4);
    for c in b.chunks(3) {
        let n =
            (u32::from(c[0]) << 16) | (u32::from(*c.get(1).unwrap_or(&0)) << 8) | u32::from(*c.get(2).unwrap_or(&0));
        for i in 0..4 {
            if i <= c.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// type.bytes.13: the bytes of standard base64 text; padding is required
/// and no other character is allowed.
pub(super) fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.as_bytes();
    if s.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    for (k, q) in s.chunks(4).enumerate() {
        let last = (k + 1) * 4 == s.len();
        let pad = q.iter().rev().take_while(|c| **c == b'=').count();
        if pad > 2 || (pad > 0 && !last) {
            return None;
        }
        let mut n = 0u32;
        for c in &q[..4 - pad] {
            n = (n << 6) | B64.iter().position(|x| x == c)? as u32;
        }
        n <<= 6 * pad as u32;
        out.extend_from_slice(&n.to_be_bytes()[1..4 - pad]);
    }
    Some(out)
}

/// A `u8` value.
pub(super) fn byte(b: u8) -> Value {
    Value::Num(Num::U8, i64::from(b))
}

/// The text form of `bytes` inside another value (type.bytes.2): `b"..."`.
pub(super) fn bytes_text(b: &[u8]) -> String {
    let mut out = String::from("b\"");
    for &c in b {
        match c {
            b'"' | b'\\' => {
                out.push('\\');
                out.push(c as char);
            }
            b'\n' => out.push_str("\\n"),
            b'\t' => out.push_str("\\t"),
            b'\r' => out.push_str("\\r"),
            0x20..=0x7e => out.push(c as char),
            _ => {
                let _ = write!(out, "\\x{c:02x}");
            }
        }
    }
    out.push('"');
    out
}

pub(super) fn bytes_value(v: Vec<u8>) -> Value {
    Value::Bytes(Rc::new(v))
}

/// The first position of `needle` in `hay` (type.bytes.9).
fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

/// type.bytes.4: `lo..hi` of `b`, trapping out of bounds.
pub(super) fn slice(b: &[u8], lo: i64, hi: i64, inclusive: bool, pos: Pos) -> Result<Value, Trap> {
    let end = if inclusive { hi.checked_add(1) } else { Some(hi) };
    let dots = if inclusive { "..=" } else { ".." };
    match (usize::try_from(lo), end.and_then(|e| usize::try_from(e).ok())) {
        (Ok(a), Some(e)) if a <= e && e <= b.len() => Ok(bytes_value(b[a..e].to_vec())),
        _ => {
            trap("type.bytes.4", pos, format!("slice {lo}{dots}{hi} is out of bounds for bytes of length {}", b.len()))
        }
    }
}

/// type.bytes.3: `b[i]`.
pub(super) fn at(b: &[u8], i: i64, pos: Pos) -> Result<Value, Trap> {
    match usize::try_from(i).ok().and_then(|u| b.get(u)) {
        Some(x) => Ok(byte(*x)),
        None => trap("type.bytes.3", pos, format!("index {i} is out of bounds for bytes of length {}", b.len())),
    }
}

/// A method of `bytes` (type.bytes.7-12); none changes the receiver.
pub(super) fn method(b: &Rc<Vec<u8>>, name: &str, a: &mut [Value]) -> Value {
    let arg = |i: usize| match &a[i] {
        Value::Bytes(x) => x.clone(),
        other => unreachable!("checked: bytes, found {other:?}"),
    };
    match name {
        "len" => Value::Int(b.len() as i64),
        "to_str" => match std::str::from_utf8(b) {
            Ok(s) => Value::Str(Rc::from(s)),
            Err(_) => Value::None,
        },
        "text" => Value::Str(Rc::from(String::from_utf8_lossy(b).as_ref())),
        "to_list" => Value::List(Rc::new(b.iter().map(|x| byte(*x)).collect())),
        "concat" => {
            let mut v = Vec::with_capacity(b.len() + arg(0).len());
            v.extend_from_slice(b);
            v.extend_from_slice(&arg(0));
            bytes_value(v)
        }
        "find" => find(b, &arg(0)).map_or(Value::None, |i| Value::Int(i as i64)),
        "contains" => Value::Bool(find(b, &arg(0)).is_some()),
        "starts_with" => Value::Bool(b.starts_with(&arg(0))),
        "ends_with" => Value::Bool(b.ends_with(&arg(0))),
        "hex" => {
            let hex = hex_encode(b);
            // type.bytes.14: with a separator, between the byte pairs.
            match a.first() {
                Some(Value::Str(sep)) => {
                    let pairs: Vec<&str> = (0..hex.len()).step_by(2).map(|i| &hex[i..i + 2]).collect();
                    Value::Str(Rc::from(pairs.join(sep)))
                }
                _ => Value::Str(Rc::from(hex)),
            }
        }
        "base64" => Value::Str(Rc::from(base64_encode(b))),
        _ => unreachable!("checked: no bytes method `{name}`"),
    }
}

impl<'a, W: Write> Interp<'a, W> {
    /// `bytes.NAME(args)`: `from_list`, `from_hex`, `from_base64` and `zeros` (type.bytes.13).
    pub(super) fn bytes_assoc(&mut self, name: &str, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let v = self.eval(&args[0].value, env)?;
        Ok(match (name, v) {
            ("from_list", Value::List(xs)) => {
                let mut out = Vec::with_capacity(xs.len());
                for x in xs.iter() {
                    out.push(match x {
                        Value::Num(_, n) | Value::Int(n) => *n as u8,
                        other => unreachable!("checked: a u8, found {other:?}"),
                    });
                }
                bytes_value(out)
            }
            ("from_hex", Value::Str(s)) => hex_decode(&s).map_or(Value::None, bytes_value),
            ("from_base64", Value::Str(s)) => base64_decode(&s).map_or(Value::None, bytes_value),
            ("zeros", Value::Int(n)) => {
                if !(0..=(1 << 32)).contains(&n) {
                    trap::<()>("type.bytes.13", pos, format!("`bytes.zeros` needs a length from 0 to 2^32, not {n}"))?;
                }
                bytes_value(vec![0; n as usize])
            }
            (n, v) => unreachable!("checked: bytes.{n}({v:?})"),
        })
    }
}
