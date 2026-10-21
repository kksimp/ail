//! Seeded input generators for the module fuzzers (fuzz.mod.1-2): adversarial
//! text, JSON, bytes (as hex), regex patterns, dates and HTTP requests. Every
//! function is a pure function of the seed, so a finding reproduces from it.

use super::progs::Rng;

/// One fuzz input: a primary and a secondary text, and an integer.
pub struct Input {
    pub text: String,
    pub text2: String,
    pub int: i64,
}

pub const KINDS: [&str; 9] = ["text", "json", "bytes", "regex", "date", "http", "compress", "csv", "args"];
pub const KINDS: [&str; 9] = ["text", "json", "bytes", "regex", "date", "http", "compress", "cookie", "httpresp"];

pub fn generate(kind: &str, seed: u64) -> Input {
    let mut r = Rng::new(seed ^ 0x5EED_F00D);
    let int = int_edge(&mut r);
    let (text, text2) = match kind {
        "json" => (json_text(&mut r), json_text(&mut r)),
        "bytes" => (bytes_hex(&mut r), bytes_hex(&mut r)),
        "compress" => (plain(&mut r), compress_hex(&mut r)),
        "regex" => {
            let pat = regex(&mut r, 0);
            let hay = haystack(&mut r, &pat);
            (if r.chance(1, 12) { corrupt(&mut r, &pat, "()[]{}*+?|\\^$.-") } else { pat }, hay)
        }
        "date" => (date_text(&mut r), date_text(&mut r)),
        "http" => (http_text(&mut r), http_text(&mut r)),
        "csv" => (csv_text(&mut r), csv_text(&mut r)),
        "args" => (args_text(&mut r), args_text(&mut r)),
        "cookie" => (cookie_text(&mut r), cookie_text(&mut r)),
        "httpresp" => (response_text(&mut r), plain(&mut r)),
        _ => (plain(&mut r), plain(&mut r)),
    };
    Input { text, text2, int }
}

fn int_edge(r: &mut Rng) -> i64 {
    const E: [i64; 14] = [0, 1, -1, 255, 256, 65535, 1 << 31, -(1 << 31), (1 << 53) + 1, i64::MAX, i64::MIN, i64::MAX - 1, 86399, 253402300799];
    if r.chance(1, 2) { *r.pick(&E) } else { (r.next() as i64) >> r.below(60) }
}

/// An AIL string literal for `s` (lex.str.1): braces, quotes and controls escaped.
pub fn literal(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '{' => out.push_str("\\{"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if (c as u32) < 0x20 || (c as u32) == 0x7f || (0xd800..0xe000).contains(&(c as u32)) => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

const PIECES: [&str; 18] = ["a", "b", "ab", "hello", " ", "  ", "\t", "\n", "\r\n", "é", "日本語", "😀", "e\u{301}", "\u{200b}", "{", "}", "\\", "\""];

fn plain(r: &mut Rng) -> String {
    let n = r.below(14);
    let mut s = String::new();
    for _ in 0..n {
        s.push_str(r.pick(&PIECES));
    }
    if r.chance(1, 10) {
        s = s.repeat(1 + r.below(600));
    }
    s
}

/// Delete, insert, swap or truncate a few characters from `alphabet`.
fn corrupt(r: &mut Rng, s: &str, alphabet: &str) -> String {
    let mut cs: Vec<char> = s.chars().collect();
    let alpha: Vec<char> = alphabet.chars().collect();
    for _ in 0..1 + r.below(3) {
        if cs.is_empty() {
            cs.push(*r.pick(&alpha));
            continue;
        }
        let i = r.below(cs.len());
        match r.below(4) {
            0 => {
                cs.remove(i);
            }
            1 => cs.insert(i, *r.pick(&alpha)),
            2 => {
                let j = r.below(cs.len());
                cs.swap(i, j);
            }
            _ => cs.truncate(i),
        }
    }
    cs.into_iter().collect()
}

// ---- JSON ----

fn json_value(r: &mut Rng, depth: usize, out: &mut String) {
    let leaf = depth >= 5 || r.chance(2, 5);
    match if leaf { r.below(5) } else { 5 + r.below(2) } {
        0 => out.push_str("null"),
        1 => out.push_str(if r.chance(1, 2) { "true" } else { "false" }),
        2 => out.push_str(r.pick(&["0", "-0", "1", "-17", "3.14159", "1e3", "2E-5", "1e999", "-1e999", "123456789012345678901234567890", "0.1e1", "9007199254740993"])),
        3 => json_string(r, out),
        4 => out.push_str(&r.below(1000).to_string()),
        5 => {
            out.push('[');
            for i in 0..r.below(5) {
                if i > 0 {
                    out.push(',');
                }
                json_value(r, depth + 1, out);
            }
            out.push(']');
        }
        _ => {
            out.push('{');
            for i in 0..r.below(5) {
                if i > 0 {
                    out.push(',');
                }
                json_string(r, out);
                out.push(':');
                json_value(r, depth + 1, out);
            }
            out.push('}');
        }
    }
}

fn json_string(r: &mut Rng, out: &mut String) {
    out.push('"');
    for _ in 0..r.below(6) {
        out.push_str(r.pick(&["a", "key", "é", "日", "😀", "\\n", "\\\"", "\\\\", "\\/", "\\u0041", "\\ud83d\\ude00", "\\ud800", "\\u0000", " "]));
    }
    out.push('"');
}

fn json_text(r: &mut Rng) -> String {
    if r.chance(1, 25) {
        let d = 100 + r.below(20000);
        return "[".repeat(d) + &"]".repeat(if r.chance(1, 2) { d } else { d / 2 });
    }
    let mut s = String::new();
    json_value(r, 0, &mut s);
    if r.chance(1, 3) {
        s = corrupt(r, &s, "{}[],:\"\\ \t\nntf0e-+.");
    }
    if r.chance(1, 8) {
        s = format!(" {s} \n");
    }
    s
}

// ---- bytes, compression ----

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn random_bytes(r: &mut Rng, n: usize) -> Vec<u8> {
    (0..n).map(|_| r.next() as u8).collect()
}

fn bytes_hex(r: &mut Rng) -> String {
    let n = *r.pick(&[0, 1, 2, 3, 4, 7, 8, 15, 16, 31, 32, 63, 64, 255, 256, 1000]);
    let b = match r.below(4) {
        0 => random_bytes(r, n),
        1 => vec![*r.pick(&[0u8, 0xff, 0x80, 0x7f, b'a'])  ; n],
        2 => (0..n).map(|i| (i % 251) as u8).collect(),
        _ => "text bytes é 日本\n".bytes().cycle().take(n).collect(),
    };
    hex(&b)
}

fn compress_hex(r: &mut Rng) -> String {
    let n = *r.pick(&[0, 1, 5, 40, 300, 5000]);
    let mut b: Vec<u8> = match r.below(4) {
        0 => random_bytes(r, n),
        1 => vec![b'x'; n],
        2 => "the quick brown fox jumps over the lazy dog. ".bytes().cycle().take(n).collect(),
        _ => (0..n).map(|i| ((i * i) >> 3) as u8).collect(),
    };
    // Headers and magics make the decoders' own paths reachable; damage stresses them.
    let magic: &[u8] = *r.pick(&[&[][..], &[0x1f, 0x8b, 0x08][..], &[0x78, 0x9c][..], &[0x50, 0x4b, 0x03, 0x04][..], &[0x50, 0x4b, 0x05, 0x06][..]]);
    let mut all = magic.to_vec();
    all.append(&mut b);
    if !all.is_empty() && r.chance(1, 3) {
        let i = r.below(all.len());
        all[i] ^= 1 << r.below(8);
    }
    if r.chance(1, 4) {
        let cut = r.below(all.len() + 1);
        all.truncate(cut);
    }
    hex(&all)
}

// ---- regex ----

fn regex(r: &mut Rng, depth: usize) -> String {
    let mut s = String::new();
    if depth == 0 && r.chance(1, 8) {
        s.push('^');
    }
    for _ in 0..1 + r.below(4) {
        let atom = match if depth >= 3 { r.below(4) } else { r.below(7) } {
            0 => r.pick(&["a", "b", "x", "é", "日", "0", " "]).to_string(),
            1 => ".".to_string(),
            2 => r.pick(&["[a-c]", "[^a]", "[a-zA-Z0-9_]", "[\\d\\s]", "[]a]", "[a-]", "[é-ü]"]).to_string(),
            3 => r.pick(&["\\d", "\\w", "\\s", "\\D", "\\W", "\\S", "\\.", "\\\\", "\\b", "\\n"]).to_string(),
            4 => format!("({})", regex(r, depth + 1)),
            5 => format!("(?:{}|{})", regex(r, depth + 1), regex(r, depth + 1)),
            _ => format!("({}|{})", regex(r, depth + 1), regex(r, depth + 1)),
        };
        s.push_str(&atom);
        if r.chance(2, 5) {
            s.push_str(r.pick(&["*", "+", "?", "*?", "+?", "{2}", "{1,3}", "{0,}", "{2,}", "{0,1000}", "{1000,2000}"]));
        }
    }
    if depth == 0 && r.chance(1, 8) {
        s.push('$');
    }
    if depth == 0 && r.chance(1, 6) {
        // The classic exponential shapes for backtracking engines (a linear-time engine must stay fast).
        s = r.pick(&["(a*)*b", "(a|aa)+$", "(a+)+b", "(.*a){12}", "(x+x+)+y", "(a?){30}a{30}"]).to_string();
    }
    s
}

fn haystack(r: &mut Rng, pat: &str) -> String {
    let alphabet: Vec<char> = pat.chars().filter(|c| c.is_alphanumeric()).collect();
    let n = *r.pick(&[0, 1, 5, 30, 200, 3000]);
    (0..n).map(|_| if alphabet.is_empty() || r.chance(1, 5) { *r.pick(&['a', 'b', ' ', 'é', '\n', '0']) } else { *r.pick(&alphabet) }).collect()
}

// ---- dates ----

fn date_text(r: &mut Rng) -> String {
    let year = *r.pick(&["2026", "1970", "0001", "0000", "9999", "10000", "-0001", "+2026", "1900", "2000", "2024"]);
    let month = *r.pick(&["01", "02", "12", "00", "13", "1", "99"]);
    let day = *r.pick(&["01", "28", "29", "30", "31", "32", "00", "5"]);
    let hour = *r.pick(&["00", "12", "23", "24", "25", "7"]);
    let min = *r.pick(&["00", "59", "60", "61"]);
    let sec = *r.pick(&["00", "59", "60", "61", "59.999999999", "00.5"]);
    let zone = *r.pick(&["Z", "+00:00", "-05:00", "+14:00", "+99:99", "+0530", "z", "", "UTC", "America/New_York", "Europe/London", "Nowhere/Land", "+5"]);
    let s = match r.below(6) {
        0 => format!("{year}-{month}-{day}"),
        1 => format!("{year}-{month}-{day}T{hour}:{min}:{sec}{zone}"),
        2 => format!("{year}{month}{day}T{hour}{min}{sec}{zone}"),
        3 => format!("{hour}:{min}:{sec}"),
        4 => r.pick(&["P1Y2M3DT4H5M6S", "PT0S", "P-1D", "PT1.5S", "P", "PT", "P1W", "1h30m", "90m", "-5s", "1d2h"]).to_string(),
        _ => format!("{day}/{month}/{year} {hour}:{min}"),
    };
    if r.chance(1, 4) { corrupt(r, &s, "0123456789-:TZ+ .") } else { s }
}

// ---- CSV and command lines (std.csv, std.args) ----

/// CSV made of fields that are plain, quoted, doubled-quoted, empty or hold a
/// line break, joined by commas and `\n`, `\r\n` or `\r`, sometimes cut short.
fn csv_text(r: &mut Rng) -> String {
    const FIELDS: [&str; 16] = [
        "a", "b c", "", "1", "-2.5", "\"q\"", "\"a,b\"", "\"two\nlines\"", "\"he said \"\"hi\"\"\"", "\"\"", "\" x\"", "x\"y", "é", "日本", "\"unterminated", "\"cr\r\nlf\"",
    ];
    let mut s = String::new();
    for _ in 0..r.below(6) {
        for k in 0..r.below(5) {
            if k > 0 {
                s.push(',');
            }
            s.push_str(r.pick(&FIELDS));
        }
        s.push_str(r.pick(&["\n", "\r\n", "\r", "\n\n", ""]));
    }
    if r.chance(1, 6) {
        let cut = r.below(s.chars().count() + 1);
        s = s.chars().take(cut).collect();
    }
    if r.chance(1, 12) {
        s = s.repeat(1 + r.below(200));
// ---- cookies and responses (std.net.jar, the HTTP/1.1 client) ----
/// A `Set-Cookie` line: name, value, and attributes in good and bad forms.
fn cookie_text(r: &mut Rng) -> String {
    let mut s = String::from(*r.pick(&["a=b", "sid=1", "=x", "x", "", "a=b=c", "a = b", "__Host-id=1", "é=ü", "a=\"q\"", "a=;", "n=v\r\nX: y"]));
    for _ in 0..r.below(5) {
        s.push_str("; ");
        s.push_str(r.pick(&[
            "Path=/", "Path=/a/b", "Path=", "Path=a", "Domain=example.com", "Domain=.example.com", "Domain=com", "Domain=", "Domain=other.org",
            "Max-Age=60", "Max-Age=0", "Max-Age=-1", "Max-Age=99999999999999999999", "Max-Age=x", "Expires=Wed, 21 Oct 2015 07:28:00 GMT",
            "Expires=Wed, 21-Oct-15 07:28:00 GMT", "Expires=Wed Oct 21 07:28:00 2015", "Expires=garbage", "Expires=Fri, 31 Dec 9999 23:59:59 GMT",
            "Secure", "HttpOnly", "SameSite=Lax", "SameSite=None", "SameSite=Strict", "SameSite=x", "secure=1", "Unknown=1", "",
        ]));
    }
    s
}

/// Command-line words, one per line: flags, options, `--`, `-`, `=` forms and junk.
fn args_text(r: &mut Rng) -> String {
    const WORDS: [&str; 22] = [
        "-v", "--verbose", "--out", "--out=x", "--out=", "-o", "x", "--", "-", "---", "--bogus", "-vo", "-ov", "file", "", "--verbose=1", "-o=x", "-h-", "--=", "é", "-9", "a b",
    ];
    let mut words = Vec::new();
    for _ in 0..r.below(8) {
        words.push(*r.pick(&WORDS));
    }
    words.join("\n")
/// A raw HTTP/1.1 response a hostile server might send.
fn response_text(r: &mut Rng) -> String {
    let status = *r.pick(&["HTTP/1.1 200 OK", "HTTP/1.1 404 Not Found", "HTTP/1.0 200 OK", "HTTP/1.1 204 No Content", "HTTP/1.1 301 Moved", "HTTP/1.1 100 Continue", "HTTP/1.1 999 X", "HTTP/1.1 20 OK", "HTTP/2 200", "HTTP/1.1", "200 OK", "", "HTTP/1.1 200", "HTTP/1.1 abc OK"]);
    let mut s = format!("{status}\r\n");
    let chunked = r.chance(2, 5);
    for _ in 0..r.below(6) {
        s.push_str(r.pick(&[
            "Content-Type: text/plain", "Content-Length: 5", "Content-Length: 0", "Content-Length: -1", "Content-Length: 99999999999", "Content-Length: 5, 5",
            "Set-Cookie: a=b; Path=/", "Set-Cookie: =", "Location: /x", "Location: http://127.0.0.1:1/", "Connection: close", "Connection: keep-alive",
            "Content-Encoding: gzip", "Content-Encoding: deflate", "Content-Encoding: br", "Transfer-Encoding: chunked", "Transfer-Encoding: gzip",
            "no-colon", " folded: x", "X: a\u{0}b", "Host:", "Trailer: x",
        ]));
        s.push_str(if r.chance(1, 10) { "\n" } else { "\r\n" });
    if chunked {
        s.push_str("Transfer-Encoding: chunked\r\n");
    s.push_str(if r.chance(1, 12) { "" } else { "\r\n" });
    if chunked {
        for _ in 0..r.below(4) {
            s.push_str(r.pick(&["5\r\nhello\r\n", "0\r\n", "1\r\nxx\r\n", "g\r\nabc\r\n", "FFFFFFFFFFFFFFFFF\r\n", "-1\r\n", "3;ext=1\r\nabc\r\n", "3\nabc\n", "", "\r\n", "5\r\nhi"]));
        }
        if r.chance(2, 3) {
            s.push_str(r.pick(&["0\r\n\r\n", "0\r\nX-T: 1\r\n\r\n", "0\r\n"]));
        }
    } else {
        s.push_str(r.pick(&["hello", "", "hello world, longer than five", "\u{1f600}", "\r\n\r\n", "\u{1f}\u{8b}\u{8}garbage"]));
    s
}

// ---- HTTP ----

fn http_text(r: &mut Rng) -> String {
    let method = *r.pick(&["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH", "get", "G E T", "", "TRACE", "CONNECT"]);
    let path = *r.pick(&["/", "/a/b?x=1&y=%20", "/%zz", "/é", "*", "http://h/x", "", "/a b", "//", "/../etc/passwd"]);
    let version = *r.pick(&["HTTP/1.1", "HTTP/1.0", "HTTP/2", "HTTP/9.9", "HTTP/1.", "http/1.1", ""]);
    let mut s = format!("{method} {path} {version}\r\n");
    for _ in 0..r.below(6) {
        let h = *r.pick(&[
            "Host: example.com",
            "Content-Length: 5",
            "Content-Length: -1",
            "Content-Length: 99999999999999999999",
            "Content-Length: 5, 5",
            "Transfer-Encoding: chunked",
            "Transfer-Encoding: chunked, gzip",
            "Connection: close",
            "Cookie: a=b; c=d",
            "X-Empty:",
            "no-colon-here",
            " folded: continuation",
            "Host: a\u{0}b",
            "Content-Type: text/plain; charset=\"utf-8\"",
            "X-Long: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ]);
        s.push_str(h);
        s.push_str(if r.chance(1, 8) { "\n" } else { "\r\n" });
    }
    s.push_str("\r\n");
    match r.below(4) {
        0 => s.push_str("hello"),
        1 => s.push_str("5\r\nhello\r\n0\r\n\r\n"),
        2 => s.push_str("zz\r\nxx\r\n"),
        _ => {}
    }
    if r.chance(1, 5) { corrupt(r, &s, "\r\n: /?&=%") } else { s }
}
