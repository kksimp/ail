//! A strict DER reader (X.690 §10) for certificates: definite, minimal
//! lengths; single-byte tags; nothing left over.

/// A parse failure; the X.509 layer turns it into a sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bad(pub &'static str);

pub type R<T> = Result<T, Bad>;

pub const BOOLEAN: u8 = 0x01;
pub const INTEGER: u8 = 0x02;
pub const BIT_STRING: u8 = 0x03;
pub const OCTET_STRING: u8 = 0x04;
pub const NULL: u8 = 0x05;
pub const OID: u8 = 0x06;
pub const UTC_TIME: u8 = 0x17;
pub const GENERALIZED_TIME: u8 = 0x18;
pub const SEQUENCE: u8 = 0x30;
pub const SET: u8 = 0x31;

/// Context-specific tags: `[n]` constructed (EXPLICIT or a constructed
/// IMPLICIT type) and primitive.
pub const fn ctx(n: u8) -> u8 {
    0xa0 | n
}
pub const fn ctx_prim(n: u8) -> u8 {
    0x80 | n
}

/// A cursor over DER elements.
#[derive(Clone, Copy)]
pub struct Der<'a> {
    data: &'a [u8],
}

impl<'a> Der<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Der { data }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The next element's tag without consuming it.
    pub fn peek(&self) -> Option<u8> {
        self.data.first().copied()
    }

    /// The next element: (tag, contents, whole encoding).
    pub fn any(&mut self) -> R<(u8, &'a [u8], &'a [u8])> {
        let d = self.data;
        let tag = *d.first().ok_or(Bad("truncated element"))?;
        if tag & 0x1f == 0x1f {
            return Err(Bad("high tag numbers aren't used in certificates"));
        }
        let first = *d.get(1).ok_or(Bad("truncated length"))?;
        let (len, head) = if first < 0x80 {
            (first as usize, 2)
        } else {
            let n = (first & 0x7f) as usize;
            if n == 0 {
                return Err(Bad("indefinite length isn't DER"));
            }
            if n > 4 {
                return Err(Bad("length too large"));
            }
            let bytes = d.get(2..2 + n).ok_or(Bad("truncated length"))?;
            if bytes[0] == 0 {
                return Err(Bad("non-minimal length"));
            }
            let len = bytes.iter().fold(0usize, |a, &b| a << 8 | b as usize);
            if len < 0x80 {
                return Err(Bad("non-minimal length"));
            }
            (len, 2 + n)
        };
        let end = head.checked_add(len).ok_or(Bad("length too large"))?;
        let whole = d.get(..end).ok_or(Bad("element runs past its container"))?;
        self.data = &d[end..];
        Ok((tag, &whole[head..], whole))
    }

    /// The next element, which must have tag `tag`; returns its contents.
    pub fn expect(&mut self, tag: u8) -> R<&'a [u8]> {
        let (t, body, _) = self.any()?;
        if t != tag {
            return Err(Bad("unexpected element"));
        }
        Ok(body)
    }

    /// Like `expect`, returning the whole encoding too.
    pub fn expect_whole(&mut self, tag: u8) -> R<(&'a [u8], &'a [u8])> {
        let (t, body, whole) = self.any()?;
        if t != tag {
            return Err(Bad("unexpected element"));
        }
        Ok((body, whole))
    }

    /// A nested reader over the contents of a `tag` element.
    pub fn nested(&mut self, tag: u8) -> R<Der<'a>> {
        Ok(Der::new(self.expect(tag)?))
    }

    /// An optional element with `tag`.
    pub fn optional(&mut self, tag: u8) -> R<Option<&'a [u8]>> {
        if self.peek() == Some(tag) { Ok(Some(self.expect(tag)?)) } else { Ok(None) }
    }

    /// A non-negative INTEGER's magnitude (leading zero removed).
    pub fn uint(&mut self) -> R<&'a [u8]> {
        let b = self.expect(INTEGER)?;
        match b {
            [] => Err(Bad("empty integer")),
            [0] => Ok(b),
            [0, next, ..] if *next < 0x80 => Err(Bad("non-minimal integer")),
            [0, rest @ ..] => Ok(rest),
            [first, ..] if *first >= 0x80 => Err(Bad("negative integer")),
            _ => Ok(b),
        }
    }

    pub fn boolean(&mut self) -> R<bool> {
        match self.expect(BOOLEAN)? {
            [0x00] => Ok(false),
            [0xff] => Ok(true),
            _ => Err(Bad("bad boolean")),
        }
    }

    /// A BIT STRING with no unused bits (keys and signatures).
    pub fn bit_string_bytes(&mut self) -> R<&'a [u8]> {
        match self.expect(BIT_STRING)? {
            [0, rest @ ..] => Ok(rest),
            _ => Err(Bad("bit string with unused bits")),
        }
    }

    pub fn finish(&self) -> R<()> {
        if self.data.is_empty() { Ok(()) } else { Err(Bad("trailing data")) }
    }
}

/// Seconds since 1970-01-01 for a UTCTime or GeneralizedTime in the forms
/// RFC 5280 §4.1.2.5 allows (`YYMMDDHHMMSSZ`, `YYYYMMDDHHMMSSZ`).
pub fn time(tag: u8, body: &[u8]) -> R<i64> {
    let digits = |s: &[u8]| -> R<i64> {
        s.iter().try_fold(0i64, |a, &c| if c.is_ascii_digit() { Ok(a * 10 + (c - b'0') as i64) } else { Err(Bad("bad time")) })
    };
    let (year, rest) = match (tag, body.len()) {
        (UTC_TIME, 13) => {
            let yy = digits(&body[..2])?;
            (if yy >= 50 { 1900 + yy } else { 2000 + yy }, &body[2..])
        }
        (GENERALIZED_TIME, 15) => (digits(&body[..4])?, &body[4..]),
        _ => return Err(Bad("bad time")),
    };
    if rest[10] != b'Z' {
        return Err(Bad("time not in UTC"));
    }
    let (mo, d, h, mi, s) =
        (digits(&rest[0..2])?, digits(&rest[2..4])?, digits(&rest[4..6])?, digits(&rest[6..8])?, digits(&rest[8..10])?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || s > 60 {
        return Err(Bad("bad time"));
    }
    Ok(days_from_civil(year, mo, d) * 86400 + h * 3600 + mi * 60 + s)
}

/// Days since 1970-01-01 (Howard Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// `YYYY-MM-DD` for messages.
pub fn date(secs: i64) -> String {
    let z = secs.div_euclid(86400) + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths_are_strict() {
        assert_eq!(Der::new(&[0x04, 0x02, 1, 2]).expect(OCTET_STRING).unwrap(), &[1, 2]);
        assert!(Der::new(&[0x04, 0x81, 0x02, 1, 2]).any().is_err()); // non-minimal
        assert!(Der::new(&[0x04, 0x80, 1, 2, 0, 0]).any().is_err()); // indefinite
        assert!(Der::new(&[0x04, 0x03, 1, 2]).any().is_err()); // short
        let mut long = vec![0x04, 0x81, 0x80];
        long.extend([7u8; 0x80]);
        assert_eq!(Der::new(&long).expect(OCTET_STRING).unwrap().len(), 0x80);
    }

    #[test]
    fn integers() {
        assert_eq!(Der::new(&[2, 2, 0, 0x80]).uint().unwrap(), &[0x80]);
        assert!(Der::new(&[2, 2, 0, 0x7f]).uint().is_err());
        assert!(Der::new(&[2, 1, 0x80]).uint().is_err());
        assert_eq!(Der::new(&[2, 1, 0]).uint().unwrap(), &[0]);
    }

    #[test]
    fn times() {
        assert_eq!(time(UTC_TIME, b"700101000000Z").unwrap(), 0);
        assert_eq!(time(UTC_TIME, b"491231235959Z").unwrap(), 2524607999);
        assert_eq!(time(GENERALIZED_TIME, b"20261020133000Z").unwrap(), 1792503000);
        assert_eq!(date(1792503000), "2026-10-20");
        assert_eq!(date(951782400), "2000-02-29");
        assert!(time(UTC_TIME, b"701301000000Z").is_err());
        assert!(time(UTC_TIME, b"700101000000+0100").is_err());
    }
}
