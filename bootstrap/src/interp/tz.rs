//! Time zones from the platform's tz database (std.tz.1-3, ADR 0448): a
//! TZif reader with the POSIX rule of its footer for instants after the
//! last transition. No crate reads the files: the format is small (RFC 9636).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// One zone's rules.
struct Zone {
    /// UTC offset in seconds for each local-time type.
    offsets: Vec<i64>,
    /// Instants (seconds) at which the type changes, ascending.
    times: Vec<i64>,
    /// The type from `times[i]` on.
    kinds: Vec<usize>,
    /// The rule after the last transition, if the file has one.
    footer: Option<Rule>,
}

/// A POSIX TZ rule such as `EST5EDT,M3.2.0,M11.1.0`.
#[derive(Clone)]
struct Rule {
    std: i64,
    /// The DST offset, the start day and time, the end day and time.
    dst: Option<(i64, Day, i64, Day, i64)>,
}

/// A day of the year in a rule.
#[derive(Clone, Copy)]
enum Day {
    /// `Mm.w.d`: month, week (5 = last), weekday (0 = Sunday).
    Month(i64, i64, i64),
    /// `Jn`: day 1-365, February 29 never counted.
    Julian(i64),
    /// `n`: day 0-365, February 29 counted.
    Zero(i64),
}

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The year containing day number `z` (days since 1970-01-01).
fn year_of(z: i64) -> i64 {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    yoe + era * 400 + i64::from(m <= 2)
}

fn leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

impl Day {
    /// Seconds since the epoch of this day's midnight in year `y`, as if UTC.
    fn midnight(self, y: i64) -> i64 {
        let first = days_from_civil(y, 1, 1);
        let days = match self {
            Day::Zero(n) => first + n,
            Day::Julian(n) => first + n - 1 + i64::from(leap(y) && n >= 60),
            Day::Month(m, w, d) => {
                let start = days_from_civil(y, m, 1);
                let wd = (start + 4).rem_euclid(7); // 1970-01-01 was a Thursday
                let mut day = start + (d - wd).rem_euclid(7) + (w - 1) * 7;
                let next = if m == 12 { days_from_civil(y + 1, 1, 1) } else { days_from_civil(y, m + 1, 1) };
                while day >= next {
                    day -= 7;
                }
                day
            }
        };
        days * 86_400
    }
}

impl Rule {
    /// The UTC offset at instant `t` under this rule.
    fn offset(&self, t: i64) -> i64 {
        let Some((dst, start, st, end, et)) = self.dst else { return self.std };
        let y = year_of((t + self.std).div_euclid(86_400));
        let on = start.midnight(y) + st - self.std;
        let off = end.midnight(y) + et - dst;
        let in_dst = if on < off { t >= on && t < off } else { t >= on || t < off };
        if in_dst { dst } else { self.std }
    }
}

/// Parse `[+-]hh[:mm[:ss]]` into seconds.
fn clock(s: &str) -> Option<i64> {
    let (sign, rest) = match s.as_bytes().first()? {
        b'-' => (-1, &s[1..]),
        b'+' => (1, &s[1..]),
        _ => (1, s),
    };
    let mut parts = rest.split(':');
    let h: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next().map_or(Some(0), |p| p.parse().ok())?;
    let sec: i64 = parts.next().map_or(Some(0), |p| p.parse().ok())?;
    Some(sign * (h * 3600 + m * 60 + sec))
}

/// Take a zone name, plain letters or `<...>`, off the front of `s`.
fn name(s: &str) -> Option<&str> {
    if let Some(rest) = s.strip_prefix('<') {
        let end = rest.find('>')?;
        Some(&s[end + 2..])
    } else {
        let end = s.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(s.len());
        (end >= 3).then(|| &s[end..])
    }
}

/// Take an offset off the front of `s`: its text and the rest.
fn offset_text(s: &str) -> (&str, &str) {
    let end = s.find(|c: char| !(c.is_ascii_digit() || c == ':' || c == '+' || c == '-')).unwrap_or(s.len());
    (&s[..end], &s[end..])
}

fn day(s: &str) -> Option<(Day, i64)> {
    let (d, t) = s.split_once('/').map_or((s, Some(7200)), |(d, t)| (d, clock(t)));
    let day = if let Some(m) = d.strip_prefix('M') {
        let mut p = m.split('.');
        Day::Month(p.next()?.parse().ok()?, p.next()?.parse().ok()?, p.next()?.parse().ok()?)
    } else if let Some(j) = d.strip_prefix('J') {
        Day::Julian(j.parse().ok()?)
    } else {
        Day::Zero(d.parse().ok()?)
    };
    Some((day, t?))
}

fn parse_rule(s: &str) -> Option<Rule> {
    let rest = name(s)?;
    let (o, rest) = offset_text(rest);
    let std = -clock(o)?;
    if rest.is_empty() {
        return Some(Rule { std, dst: None });
    }
    let rest = name(rest)?;
    let (o, rest) = offset_text(rest);
    let dst = if o.is_empty() { std + 3600 } else { -clock(o)? };
    let mut it = rest.strip_prefix(',')?.split(',');
    let (start, st) = day(it.next()?)?;
    let (end, et) = day(it.next()?)?;
    Some(Rule { std, dst: Some((dst, start, st, end, et)) })
}

/// A big-endian signed integer of `n` bytes (4 or 8).
fn be(b: &[u8], n: usize) -> Option<i64> {
    let b = b.get(..n)?;
    let mut v: i64 = 0;
    for &x in b {
        v = (v << 8) | i64::from(x);
    }
    Some(if n == 4 { i64::from(v as u32 as i32) } else { v })
}

/// Read a TZif file (RFC 9636): the version 2+ block when there is one.
fn parse(data: &[u8]) -> Option<Zone> {
    if data.get(..4)? != b"TZif" {
        return None;
    }
    let version = data[4];
    let counts = |at: usize| -> Option<[usize; 6]> {
        let h = data.get(at + 20..at + 44)?;
        let mut c = [0; 6];
        for (i, v) in c.iter_mut().enumerate() {
            *v = usize::try_from(be(&h[i * 4..], 4)?).ok()?;
        }
        Some(c)
    };
    let c1 = counts(0)?;
    let (mut at, mut width, mut c) = (44, 4, c1);
    if version >= b'2' {
        // Skip the version 1 block.
        let [isut, isstd, leapc, time, kind, chars] = c1;
        at += time * 4 + time + kind * 6 + chars + leapc * 8 + isstd + isut;
        c = counts(at)?;
        at += 44;
        width = 8;
    }
    let [isut, isstd, leapcnt, timecnt, typecnt, charcnt] = c;
    let mut times = Vec::with_capacity(timecnt);
    for i in 0..timecnt {
        times.push(be(data.get(at + i * width..)?, width)?);
    }
    at += timecnt * width;
    let kinds: Vec<usize> = data.get(at..at + timecnt)?.iter().map(|&k| usize::from(k)).collect();
    at += timecnt;
    let mut offsets = Vec::with_capacity(typecnt);
    for i in 0..typecnt {
        offsets.push(be(data.get(at + i * 6..)?, 4)?);
    }
    at += typecnt * 6 + charcnt + leapcnt * (width + 4) + isstd + isut;
    let footer = if version >= b'2' {
        let tail = std::str::from_utf8(data.get(at..)?).ok()?;
        tail.strip_prefix('\n').and_then(|t| t.split('\n').next()).filter(|l| !l.is_empty()).and_then(parse_rule)
    } else {
        None
    };
    (!offsets.is_empty() && kinds.iter().all(|&k| k < offsets.len())).then_some(Zone { offsets, times, kinds, footer })
}

impl Zone {
    fn offset(&self, t: i64) -> i64 {
        match self.times.partition_point(|&x| x <= t) {
            0 => self.offsets[0],
            n if n == self.times.len() && self.footer.is_some() => self.footer.as_ref().map_or(0, |f| f.offset(t)),
            n => self.offsets[self.kinds[n - 1]],
        }
    }
}

const DIRS: [&str; 4] = ["/usr/share/zoneinfo", "/usr/lib/zoneinfo", "/usr/share/lib/zoneinfo", "/etc/zoneinfo"];

/// Whether `name` is a safe zone name: letters, digits and `/_+-` only, no `..`.
fn safe(name: &str) -> bool {
    !name.is_empty()
        && !name.contains("..")
        && !name.starts_with('/')
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b"/_+-".contains(&b))
}

type Cache = Mutex<HashMap<String, Option<Arc<Zone>>>>;
static CACHE: OnceLock<Cache> = OnceLock::new();

fn load(name: &str) -> Option<Arc<Zone>> {
    let cache = CACHE.get_or_init(Default::default);
    if let Some(z) = cache.lock().ok()?.get(name) {
        return z.clone();
    }
    let zone = safe(name)
        .then(|| DIRS.iter().find_map(|d| std::fs::read(format!("{d}/{name}")).ok()))
        .flatten()
        .and_then(|b| parse(&b))
        .map(Arc::new);
    cache.lock().ok()?.insert(name.to_string(), zone.clone());
    zone
}

/// std.tz.1: the UTC offset in seconds of IANA zone `name` at instant `t`
/// (seconds since the epoch), or `None` when the zone isn't installed.
pub fn offset(name: &str, t: i64) -> Option<i64> {
    load(name).map(|z| z.offset(t))
}

/// std.tz.2: the system's zone: `TZ`, else the target of `/etc/localtime`,
/// else `UTC`. Windows has no tz database files; it reports `UTC` (ADR 0448).
pub fn system() -> String {
    if let Ok(tz) = std::env::var("TZ") {
        let tz = tz.trim_start_matches(':');
        if safe(tz) {
            return tz.to_string();
        }
    }
    if let Ok(target) = std::fs::read_link("/etc/localtime")
        && let Some(s) = target.to_str()
        && let Some(i) = s.find("zoneinfo/")
    {
        return s[i + 9..].to_string();
    }
    "UTC".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_days() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(year_of(11_017), 2000);
        assert_eq!(year_of(-1), 1969);
    }

    #[test]
    fn posix_rules() {
        let r = parse_rule("EST5EDT,M3.2.0,M11.1.0").unwrap();
        // 2026-03-08 07:00:00Z is when EST becomes EDT.
        let t = days_from_civil(2026, 3, 8) * 86_400 + 7 * 3600;
        assert_eq!(r.offset(t - 1), -18_000);
        assert_eq!(r.offset(t), -14_400);
        // 2026-11-01 06:00:00Z ends DST.
        let t = days_from_civil(2026, 11, 1) * 86_400 + 6 * 3600;
        assert_eq!(r.offset(t - 1), -14_400);
        assert_eq!(r.offset(t), -18_000);
        // Southern hemisphere: DST wraps the new year.
        let r = parse_rule("<+10>-10<+11>,M10.1.0,M4.1.0/3").unwrap();
        assert_eq!(r.offset(days_from_civil(2026, 1, 15) * 86_400), 39_600);
        assert_eq!(r.offset(days_from_civil(2026, 7, 15) * 86_400), 36_000);
        assert_eq!(parse_rule("UTC0").unwrap().offset(5), 0);
    }

    #[test]
    fn unsafe_names() {
        assert!(!safe("../etc/passwd"));
        assert!(!safe("/etc/passwd"));
        assert!(safe("America/New_York"));
        assert_eq!(offset("Not/AZone", 0), None);
    }
}
