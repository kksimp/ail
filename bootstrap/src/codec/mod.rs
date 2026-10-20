//! In-house hashing and compression for `std.hash` and `std.compress`
//! (ADRs 0442-0446, ADR 0009): CRC-32, a fast 64-bit hash, DEFLATE
//! (RFC 1951), gzip (RFC 1952) and zip. Rust standard library only; SHA-2
//! and HMAC come from `net::crypto` (ADR 0366).

pub mod crc32;
pub mod deflate;
pub mod gzip;
pub mod hash64;
pub mod inflate;
pub mod zip;
pub mod zlib;

/// A failure to decode or encode compressed data: a short machine `kind`
/// and a message for people (std.compress.2).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CodecError {
    pub kind: &'static str,
    pub message: String,
}

impl CodecError {
    pub fn new(kind: &'static str, message: impl Into<String>) -> Self {
        CodecError { kind, message: message.into() }
    }
}

/// Lower-case hex of `bytes` (std.hash.2).
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(DIGITS[(b >> 4) as usize] as char);
        s.push(DIGITS[(b & 15) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::deflate::deflate;
    use super::gzip::{gunzip, gzip};
    use super::inflate::inflate;
    use super::zip;
    use std::io::Write;
    use std::process::{Command, Stdio};

    const BIG: usize = 1 << 28;

    /// A deterministic mix: runs, text-like bytes and noise.
    fn sample(n: usize, seed: u64) -> Vec<u8> {
        let mut s = seed | 1;
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            match s % 4 {
                0 => out.extend(std::iter::repeat((s >> 8) as u8).take((s >> 20) as usize % 300)),
                1 => out.extend(b"the quick brown fox jumps over the lazy dog ".iter().take((s >> 20) as usize % 44)),
                2 => out.extend((0..(s >> 20) % 64).map(|i| (s >> (i % 7)) as u8)),
                _ => out.push((s >> 33) as u8),
            }
        }
        out.truncate(n);
        out
    }

    #[test]
    fn rfc1951_streams() {
        // Fixed-Huffman streams as zlib writes them.
        assert_eq!(inflate(&[0x03, 0x00], BIG).unwrap(), b"");
        assert_eq!(inflate(&[0x4b, 0x04, 0x00], BIG).unwrap(), b"a");
        assert_eq!(inflate(&[0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x07, 0x00], BIG).unwrap(), b"hello");
        // A stored block (RFC 1951 3.2.4).
        assert_eq!(inflate(&[0x01, 0x03, 0x00, 0xfc, 0xff, b'a', b'b', b'c'], BIG).unwrap(), b"abc");
    }

    #[test]
    fn round_trips_at_every_level() {
        for (n, seed) in [(0, 1), (1, 2), (2, 3), (3, 4), (258, 5), (65535, 6), (65536, 7), (200_000, 8)] {
            let data = sample(n, seed);
            for level in [0, 1, 4, 6, 9] {
                let packed = deflate(&data, level);
                assert_eq!(inflate(&packed, BIG).unwrap(), data, "n={n} level={level}");
            }
        }
    }

    #[test]
    fn compresses_repetitive_data() {
        let data = vec![b'x'; 100_000];
        assert!(deflate(&data, 6).len() < 300);
        let noise = sample(5000, 99).iter().map(|b| b.wrapping_mul(167).wrapping_add(13)).collect::<Vec<u8>>();
        assert!(deflate(&noise, 6).len() <= noise.len() + 16);
    }

    #[test]
    fn corrupt_input_is_an_error_not_a_panic() {
        let packed = deflate(&sample(5000, 11), 6);
        for cut in [0, 1, 2, packed.len() / 2, packed.len() - 1] {
            assert!(inflate(&packed[..cut], BIG).is_err(), "cut {cut}");
        }
        for i in 0..packed.len().min(200) {
            let mut bad = packed.clone();
            bad[i] ^= 0x55;
            let _ = inflate(&bad, BIG); // must not panic
        }
        assert_eq!(inflate(&deflate(&vec![0; 10_000], 6), 100).unwrap_err().kind, "too_big");
    }

    #[test]
    fn gzip_round_trip_and_checks() {
        let data = sample(50_000, 21);
        let z = gzip(&data, 6);
        assert_eq!(gunzip(&z, BIG).unwrap(), data);
        let mut two = z.clone();
        two.extend(gzip(b"tail", 6));
        let mut both = data.clone();
        both.extend(b"tail");
        assert_eq!(gunzip(&two, BIG).unwrap(), both);
        let mut bad = z.clone();
        let n = bad.len();
        bad[n - 6] ^= 1;
        assert_eq!(gunzip(&bad, BIG).unwrap_err().kind, "checksum");
        assert!(gunzip(b"not gzip at all, nope", BIG).is_err());
    }

    fn system(cmd: &str, args: &[&str], input: &[u8]) -> Option<Vec<u8>> {
        let mut child = Command::new(cmd).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().ok()?;
        let mut stdin = child.stdin.take()?;
        let data = input.to_vec();
        let t = std::thread::spawn(move || {
            let _ = stdin.write_all(&data);
        });
        let out = child.wait_with_output().ok()?;
        t.join().ok()?;
        out.status.success().then_some(out.stdout)
    }

    #[test]
    fn agrees_with_system_gzip() {
        let Some(_) = system("gzip", &["-c"], b"probe") else {
            eprintln!("note: no system gzip; cross-check skipped (std.compress.6)");
            return;
        };
        for (n, seed) in [(0, 1), (100, 2), (70_000, 3), (400_000, 4)] {
            let data = sample(n, seed);
            // Ours, read by the system tool.
            for level in [1, 6, 9] {
                let back = system("gzip", &["-dc"], &gzip(&data, level)).expect("system gzip reads our output");
                assert_eq!(back, data, "n={n} level={level}");
            }
            // The system tool's, read by ours.
            for flag in ["-1", "-6", "-9"] {
                let theirs = system("gzip", &["-c", flag], &data).unwrap();
                assert_eq!(gunzip(&theirs, BIG).unwrap(), data, "n={n} {flag}");
            }
        }
    }

    #[test]
    fn zip_round_trip_and_safety() {
        let mut files = vec![
            zip::Entry { name: "b/x.txt".into(), data: b"hello hello hello hello hello".to_vec() },
            zip::Entry { name: "a.bin".into(), data: sample(30_000, 5) },
            zip::Entry { name: "empty".into(), data: vec![] },
        ];
        let z = zip::write(&mut files, 6).unwrap();
        assert_eq!(z, zip::write(&mut files, 6).unwrap(), "deterministic");
        let back = zip::read(&z, BIG).unwrap();
        assert_eq!(back.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["a.bin", "b/x.txt", "empty"]);
        assert_eq!(back[1].data, b"hello hello hello hello hello");
        assert_eq!(back[0].data, sample(30_000, 5));
        for bad in ["../x", "/abs", "a/../b", "a\\b", "c:x", "a//b", ""] {
            assert!(!zip::safe_name(bad), "{bad}");
        }
        let mut evil = vec![zip::Entry { name: "../escape".into(), data: b"x".to_vec() }];
        let z = zip::write(&mut evil, 6).unwrap();
        assert_eq!(zip::read(&z, BIG).unwrap_err().kind, "unsafe_path");
    }

    #[test]
    fn zip_agrees_with_system_unzip() {
        let dir = std::env::temp_dir().join(format!("ail-zip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut files = vec![
            zip::Entry { name: "d/one.txt".into(), data: b"one one one one one one one".to_vec() },
            zip::Entry { name: "two.bin".into(), data: sample(20_000, 8) },
        ];
        let path = dir.join("t.zip");
        std::fs::write(&path, zip::write(&mut files, 6).unwrap()).unwrap();
        let ok = Command::new("unzip").arg("-tq").arg(&path).output();
        match ok {
            Ok(o) => assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stdout)),
            Err(_) => eprintln!("note: no system unzip; cross-check skipped (std.compress.6)"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `cargo test --release codec::tests::throughput -- --ignored --nocapture`
    /// prints MB/s for the numbers recorded in ADR 0445.
    #[test]
    #[ignore]
    fn throughput() {
        use std::time::Instant;
        let text: Vec<u8> = {
            let src = std::fs::read_to_string("../docs/spec/17-std.md").unwrap();
            src.bytes().cycle().take(32 << 20).collect()
        };
        let mixed = sample(32 << 20, 3);
        let mb = |n: usize, t: Instant| n as f64 / 1e6 / t.elapsed().as_secs_f64();
        let t = Instant::now();
        std::hint::black_box(super::crc32::crc32(&text));
        println!("crc32      {:8.0} MB/s", mb(text.len(), t));
        let t = Instant::now();
        std::hint::black_box(super::hash64::hash64(&text));
        println!("hash64     {:8.0} MB/s", mb(text.len(), t));
        use crate::net::crypto::sha2::{Hash, Sha256, Sha512};
        let t = Instant::now();
        std::hint::black_box(Sha256::digest(&text));
        println!("sha256     {:8.0} MB/s", mb(text.len(), t));
        let t = Instant::now();
        std::hint::black_box(Sha512::digest(&text));
        println!("sha512     {:8.0} MB/s", mb(text.len(), t));
        for (name, data) in [("text", &text), ("mixed", &mixed)] {
            for level in [1, 6, 9] {
                let t = Instant::now();
                let z = super::deflate::deflate(data, level);
                let c = mb(data.len(), t);
                let t = Instant::now();
                std::hint::black_box(inflate(&z, BIG).unwrap());
                println!(
                    "{name:5} L{level} deflate {c:6.0} MB/s  inflate {:6.0} MB/s  ratio {:.3}",
                    mb(data.len(), t),
                    z.len() as f64 / data.len() as f64
                );
            }
        }
    }
}
