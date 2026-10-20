//! The vector file `tests/regex/vectors.txt` (std.regex.3-8): searches,
//! all-matches, replacements, splits and compile errors, run against the engine.

use super::*;

fn unescape(s: &str) -> String {
    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('f') => out.push('\x0C'),
            Some('v') => out.push('\x0B'),
            Some('a') => out.push('\x07'),
            Some('\\') => out.push('\\'),
            Some('u') => {
                let hex: String = (0..4).filter_map(|_| it.next()).collect();
                out.push(char::from_u32(u32::from_str_radix(&hex, 16).unwrap()).unwrap());
            }
            other => panic!("bad escape {other:?} in {s:?}"),
        }
    }
    out
}

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

fn spans(ms: &[Caps]) -> String {
    if ms.is_empty() {
        return "-".into();
    }
    ms.iter().map(|m| format!("{}-{}", m[0], m[1])).collect::<Vec<_>>().join(" ")
}

#[test]
fn vectors() {
    let file = include_str!("../../tests/regex/vectors.txt");
    let mut n = 0;
    let mut bad = Vec::new();
    for line in file.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
        n += 1;
        let f: Vec<&str> = line.split('\t').collect();
        let got = match f[0] {
            "ERR" => match compile(f[1]) {
                Ok(_) => "compiled".to_string(),
                Err(e) => format!("{} {}", e.kind, e.at),
            }
            .to_string()
            .eq(&format!("{} {}", f[2], f[3]))
            .then(|| "ok".to_string())
            .unwrap_or_else(|| match compile(f[1]) {
                Ok(_) => "compiled".into(),
                Err(e) => format!("{} {}", e.kind, e.at),
            }),
            "ALL" => {
                let p = compile(f[1]).unwrap_or_else(|e| panic!("{line}: {}", e.message));
                let t = chars(&unescape(f[2]));
                let got = spans(&p.find_all(&t, 2, usize::MAX));
                if got == f[3] { "ok".into() } else { got }
            }
            "REPLACE" | "REPLACE_FIRST" => {
                let p = compile(f[1]).unwrap_or_else(|e| panic!("{line}: {}", e.message));
                let t = chars(&unescape(f[2]));
                let parts = template(f[3], &p).unwrap();
                let limit = if f[0] == "REPLACE" { usize::MAX } else { 1 };
                let got = replace(&p, &t, &parts, limit);
                if got == unescape(f[4]) { "ok".into() } else { got }
            }
            "SPLIT" => {
                let p = compile(f[1]).unwrap_or_else(|e| panic!("{line}: {}", e.message));
                let t = chars(&unescape(f[2]));
                let got = split(&p, &t).join("¦");
                if got == unescape(f[3]) { "ok".into() } else { got }
            }
            _ => {
                let p = compile(f[0]).unwrap_or_else(|e| panic!("{line}: {}", e.message));
                let t = chars(&unescape(f[1]));
                let got = match p.exec(&t, 0, 2 * p.groups, false) {
                    None => "-".to_string(),
                    Some(m) => (0..p.groups)
                        .map(|g| if m[2 * g] == UNSET { "_".to_string() } else { format!("{}-{}", m[2 * g], m[2 * g + 1]) })
                        .collect::<Vec<_>>()
                        .join(" "),
                };
                // is_match must agree with find.
                assert_eq!(p.is_match(&t), got != "-", "{line}");
                if got == f[2] { "ok".into() } else { got }
            }
        };
        if got != "ok" {
            bad.push(format!("{line}\n   got: {got}"));
        }
    }
    assert!(n >= 200, "only {n} vectors");
    assert!(bad.is_empty(), "{} of {n} vectors differ:\n{}", bad.len(), bad.join("\n"));
}
