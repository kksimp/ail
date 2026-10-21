//! Handshake tests: a recorded real handshake replayed byte for byte, a
//! live `openssl s_server` when one is installed, and refusals.

use super::*;
use crate::net::crypto::hex;
use std::cell::RefCell;

/// Accepts any certificate, recording what it was shown. Tests only: the
/// real verifier is slice 3 (ADR 0368).
#[derive(Default)]
struct AcceptAny {
    seen: RefCell<Vec<(usize, u16)>>,
}

impl Verifier for AcceptAny {
    fn verify(&self, _host: &str, chain: &[Vec<u8>], scheme: u16, signed: &[u8], _sig: &[u8]) -> Result<(), TlsError> {
        assert!(signed.starts_with(&[0x20; 64]));
        self.seen.borrow_mut().push((chain.len(), scheme));
        Ok(())
    }
}

/// A stream that plays back the server's bytes and records the client's.
struct Replay {
    input: Vec<u8>,
    pos: usize,
    written: Vec<u8>,
}

impl Read for Replay {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = buf.len().min(self.input.len() - self.pos);
        buf[..n].copy_from_slice(&self.input[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

impl Write for Replay {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn illustrated() -> (Vec<u8>, Vec<u8>) {
    let mut lines = include_str!("../testdata/illustrated-tls13.txt").lines().filter(|l| !l.starts_with('#'));
    (hex(lines.next().unwrap()), hex(lines.next().unwrap()))
}

fn illustrated_ephemeral() -> Ephemeral {
    Ephemeral { random: [0; 32], session_id: [0; 32], secret: std::array::from_fn(|i| 0x20 + i as u8) }
}

// The whole key schedule, both directions' record protection, Finished in
// both directions and post-handshake tickets, checked against a real
// handshake: every byte the client writes must equal the recording.
#[test]
fn replays_a_recorded_handshake_byte_for_byte() {
    let (client, server) = illustrated();
    let hello = client[5..5 + 248].to_vec();
    let verifier = AcceptAny::default();
    let config = Config { host: "example.ulfheim.net", verifier: &verifier };
    let replay = Replay { input: server, pos: 0, written: Vec::new() };
    let mut tls = handshake_with(replay, &config, illustrated_ephemeral(), Some(hello)).unwrap();
    assert_eq!(tls.suite(), "TLS_AES_256_GCM_SHA384");
    assert_eq!(*verifier.seen.borrow(), vec![(1, 0x0804)]);
    tls.write_all(b"ping").unwrap();
    assert_eq!(hex_of(&tls.records.stream.written), hex_of(&client));
    let mut got = Vec::new();
    tls.read_to_end(&mut got).unwrap();
    assert_eq!(got, b"pong");
}

fn hex_of(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

// Each corrupted byte of the server's flight must stop the handshake with
// an error, never a panic or a success.
#[test]
fn corrupted_server_bytes_fail_cleanly() {
    let (client, server) = illustrated();
    let hello = client[5..5 + 248].to_vec();
    let verifier = AcceptAny::default();
    let config = Config { host: "example.ulfheim.net", verifier: &verifier };
    // Up to the end of the server's Finished record.
    let flight = 5 + 122 + 6 + (5 + 23) + (5 + 835) + (5 + 281) + (5 + 69);
    for i in (0..flight).step_by(7) {
        let mut bad = server.clone();
        bad[i] ^= 0x41;
        let replay = Replay { input: bad, pos: 0, written: Vec::new() };
        let r = handshake_with(replay, &config, illustrated_ephemeral(), Some(hello.clone()));
        // A flipped byte inside the plaintext CCS record body or the legacy
        // record version may be tolerated; nothing else may succeed.
        if let Ok(_) = r {
            assert!(matches!(i, 2 | 129 | 133), "byte {i} corrupted but the handshake succeeded");
        }
    }
    // Truncation at every record boundary is an error too.
    for cut in [0, 5, 127, 133, 161, 1001] {
        let replay = Replay { input: server[..cut].to_vec(), pos: 0, written: Vec::new() };
        assert!(handshake_with(replay, &config, illustrated_ephemeral(), Some(hello.clone())).is_err());
    }
}

#[test]
fn refuses_a_tls12_server() {
    // A TLS 1.2 ServerHello: no supported_versions extension.
    let mut body = vec![3, 3];
    body.extend([7u8; 32]);
    body.push(32);
    body.extend([0u8; 32]);
    body.extend([0xc0, 0x2f, 0]);
    body.extend([0, 0]);
    let msg = handshake(SERVER_HELLO, &body);
    let mut input = vec![HANDSHAKE, 3, 3];
    input.extend((msg.len() as u16).to_be_bytes());
    input.extend(&msg);
    let verifier = AcceptAny::default();
    let config = Config { host: "old.example", verifier: &verifier };
    let e = Ephemeral { random: [1; 32], session_id: [0; 32], secret: [2; 32] };
    let err = handshake_with(Replay { input, pos: 0, written: Vec::new() }, &config, e, None).err().unwrap();
    assert_eq!(err.message, "the server only speaks TLS 1.2 or older; AIL supports TLS 1.3 only (ADR 0360)");
}

fn server_hello_record(retry: bool, extensions: &[u8]) -> Vec<u8> {
    let mut body = vec![3, 3];
    body.extend(if retry { RETRY_RANDOM } else { [7u8; 32] });
    body.push(32);
    body.extend([2u8; 32]); // echoes the session id below
    body.extend([0x13, 0x01, 0]);
    put_vec(&mut body, 2, extensions);
    let msg = handshake(SERVER_HELLO, &body);
    let mut rec = vec![HANDSHAKE, 3, 3];
    rec.extend((msg.len() as u16).to_be_bytes());
    rec.extend(&msg);
    rec
}

fn first_flight_error(input: Vec<u8>) -> String {
    let verifier = AcceptAny::default();
    let config = Config { host: "example.com", verifier: &verifier };
    let e = Ephemeral { random: [1; 32], session_id: [2; 32], secret: [3; 32] };
    handshake_with(Replay { input, pos: 0, written: Vec::new() }, &config, e, None).err().unwrap().message
}

#[test]
fn refuses_hello_retry_for_another_group() {
    // supported_versions 1.3, key_share asking for secp256r1 (0x0017).
    let ext = hex("002b 0002 0304 0033 0002 0017");
    assert_eq!(
        first_flight_error(server_hello_record(true, &ext)),
        "the server requires key exchange group 0x0017; AIL's TLS supports only X25519"
    );
    let ext = hex("002b 0002 0304 0033 0002 001d");
    assert!(first_flight_error(server_hello_record(true, &ext)).contains("already offered"));
    // A ServerHello choosing a group we never offered.
    let ext = hex("002b 0002 0304 0033 0006 0017 0002 abcd");
    assert!(first_flight_error(server_hello_record(false, &ext)).contains("wasn't offered"));
    // A low-order X25519 point (all zeros) gives an all-zero secret.
    let mut ext = hex("002b 0002 0304 0033 0024 001d 0020");
    ext.extend([0u8; 32]);
    assert!(first_flight_error(server_hello_record(false, &ext)).contains("invalid"));
}

#[test]
fn refuses_plain_http() {
    let verifier = AcceptAny::default();
    let config = Config { host: "example.com", verifier: &verifier };
    let input = b"HTTP/1.1 400 Bad Request\r\n\r\n".to_vec();
    let err = connect(Replay { input, pos: 0, written: Vec::new() }, &config).err().unwrap();
    assert!(err.message.contains("didn't answer with TLS"), "{}", err.message);
}

#[test]
fn client_hello_shape() {
    let e = Ephemeral { random: [1; 32], session_id: [2; 32], secret: [3; 32] };
    let ch = client_hello("example.com", &e, None);
    assert_eq!(ch[0], CLIENT_HELLO);
    let s = hex_of(&ch);
    assert!(s.contains("0006130313011302"), "cipher suites"); // offered, in order
    assert!(s.contains("000b6578616d706c652e636f6d"), "SNI");
    assert!(s.contains("002b0003020304"), "supported_versions");
    // No SNI for an address.
    assert!(!hex_of(&client_hello("127.0.0.1", &e, None)).contains("0000000e"));
}

/// Live handshakes against `openssl s_server`, one per cipher suite. Skips
/// (passing, with a note) when `openssl` isn't installed.
#[test]
fn live_openssl_server() {
    let Some(server) = openssl::Server::cert_dir() else {
        eprintln!("note: no OpenSSL 1.1.1+ `openssl`; skipping the live TLS handshake test");
        return;
    };
    for (suite, extra) in [
        ("TLS_CHACHA20_POLY1305_SHA256", &[][..]),
        ("TLS_AES_128_GCM_SHA256", &[][..]),
        ("TLS_AES_256_GCM_SHA384", &[][..]),
        // The server prefers P-256 but can take our X25519 share as sent.
        ("TLS_AES_128_GCM_SHA256", &["-groups", "P-256:X25519"][..]),
    ] {
        let mut args = vec!["-tls1_3", "-ciphersuites", suite];
        args.extend_from_slice(extra);
        let mut running = server.start(&args);
        let tcp = running.connect();
        let verifier = AcceptAny::default();
        let config = Config { host: "localhost", verifier: &verifier };
        let mut tls = connect(tcp, &config).unwrap_or_else(|e| panic!("{suite} {extra:?}: {e}"));
        assert_eq!(tls.suite(), suite);
        tls.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
        let mut body = Vec::new();
        tls.read_to_end(&mut body).unwrap();
        let text = String::from_utf8_lossy(&body);
        assert!(text.starts_with("HTTP/1.0 200 ok"), "{suite}: {text}");
        assert!(text.contains(suite), "the server reports the suite: {text}");
        assert_eq!(*verifier.seen.borrow(), vec![(1, 0x0403)]);
    }
    // Refusals with a reason.
    let mut running = server.start(&["-tls1_2"]);
    let verifier = AcceptAny::default();
    let config = Config { host: "localhost", verifier: &verifier };
    let err = connect(running.connect(), &config).err().unwrap();
    assert!(err.message.contains("doesn't support TLS 1.3"), "{}", err.message);
    let mut running = server.start(&["-tls1_3", "-groups", "P-256"]);
    let err = connect(running.connect(), &config).err().unwrap();
    assert!(err.message.contains("handshake_failure"), "{}", err.message);
}

/// The server rolls its keys and asks us to roll ours (§4.6.3): typing `K`
/// into `openssl s_server` sends KeyUpdate(update_requested). Data must
/// still flow both ways afterwards.
#[test]
fn live_openssl_key_update() {
    let Some(server) = openssl::Server::cert_dir() else {
        eprintln!("note: no OpenSSL 1.1.1+ `openssl`; skipping the live KeyUpdate test");
        return;
    };
    let mut running = server.start_echo(&["-tls1_3"]);
    let verifier = AcceptAny::default();
    let config = Config { host: "localhost", verifier: &verifier };
    let mut tls = connect(running.connect(), &config).unwrap();
    // Let the server finish its side of the handshake before typing.
    std::thread::sleep(std::time::Duration::from_millis(300));
    running.type_line("K");
    std::thread::sleep(std::time::Duration::from_millis(100));
    running.type_line("after the update");
    let mut got = Vec::new();
    while !got.ends_with(b"\n") {
        let mut buf = [0u8; 64];
        let n = tls.read(&mut buf).unwrap();
        assert!(n > 0, "the server closed early");
        got.extend_from_slice(&buf[..n]);
    }
    assert_eq!(got, b"after the update\n");
    tls.write_all(b"from the client\n").unwrap();
    assert!(running.wait_for_output("from the client"), "the server couldn't read data under our new keys");
}

pub(crate) mod openssl {
    use std::net::TcpStream;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    /// A directory holding a fresh P-256 key and self-signed certificate.
    pub struct Server {
        dir: PathBuf,
    }

    impl Server {
        pub fn cert_dir() -> Option<Server> {
            // OpenSSL 1.1.1 or 3.x: LibreSSL (macOS's `openssl`) lacks some of
            // the s_server options used here.
            let version = Command::new("openssl").arg("version").stderr(Stdio::null()).output().ok()?;
            let text = String::from_utf8_lossy(&version.stdout);
            if !(text.starts_with("OpenSSL 3") || text.starts_with("OpenSSL 1.1.1")) {
                eprintln!("note: `openssl` is `{}`, not OpenSSL 1.1.1 or 3", text.trim());
                return None;
            }
            // One directory per server: tests run in parallel.
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!("ail-tls-test-{}-{n}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let ok = Command::new("openssl")
                .args(["req", "-x509", "-newkey", "ec", "-pkeyopt", "ec_paramgen_curve:P-256", "-nodes"])
                .args(["-days", "1", "-subj", "/CN=localhost", "-keyout", "key.pem", "-out", "cert.pem"])
                .current_dir(&dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success();
            assert!(ok, "openssl couldn't make a test certificate");
            Some(Server { dir })
        }

        pub fn start(&self, args: &[&str]) -> Running {
            self.spawn(&["-www"], args, false)
        }

        /// Starts `s_server` listening on port 0 and learns the port it got from the
        /// `ACCEPT host:port` line it prints. The server chooses and holds the port, so
        /// there is no window between picking a free port and binding it in which another
        /// test, or another process, could take it.
        fn spawn(&self, mode: &[&str], args: &[&str], echo: bool) -> Running {
            let mut child = Command::new("openssl")
                .args(["s_server", "-key", "key.pem", "-cert", "cert.pem"])
                .args(["-accept", "127.0.0.1:0"])
                .args(mode)
                .args(args)
                .current_dir(&self.dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let mut out = child.stdout.take().unwrap();
            let (port_tx, port_rx) = std::sync::mpsc::channel();
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let mut seen = String::new();
                let mut sent_port = false;
                let mut buf = [0u8; 4096];
                // Always drain the pipe, so a chatty server never blocks on a full one.
                while let Ok(n) = std::io::Read::read(&mut out, &mut buf) {
                    if n == 0 {
                        break;
                    }
                    if !sent_port {
                        seen.push_str(&String::from_utf8_lossy(&buf[..n]));
                        if let Some(rest) = seen.split("ACCEPT 127.0.0.1:").nth(1) {
                            if let Some(end) = rest.find('\n') {
                                if let Ok(port) = rest[..end].trim().parse::<u16>() {
                                    let _ = port_tx.send(port);
                                    sent_port = true;
                                }
                            }
                        }
                    }
                    if echo && tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
            });
            Running { child, port: None, port_rx, output: echo.then_some(rx) }
        }
    }

    impl Server {
        /// A server directory holding the given key, certificate and
        /// intermediates (PEM, served with `-cert_chain chain.pem`) instead
        /// of a fresh self-signed pair.
        pub fn from_pem(key: &str, cert: &str, chain: &str) -> Option<Server> {
            let server = Server::cert_dir()?;
            std::fs::write(server.dir.join("key.pem"), key).unwrap();
            std::fs::write(server.dir.join("cert.pem"), cert).unwrap();
            std::fs::write(server.dir.join("chain.pem"), chain).unwrap();
            Some(server)
        }

        /// `s_server` without `-www`: it echoes what it reads to its stdout
        /// and sends what is typed on its stdin.
        pub fn start_echo(&self, args: &[&str]) -> Running {
            self.spawn(&[], args, true)
        }
    }

    impl Drop for Server {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    pub struct Running {
        child: Child,
        port: Option<u16>,
        port_rx: std::sync::mpsc::Receiver<u16>,
        output: Option<std::sync::mpsc::Receiver<Vec<u8>>>,
    }

    impl Running {
        /// Connects once the server reports the port it listens on (it can take a
        /// while to start on a loaded machine). The port line is printed after
        /// `listen`, so the connection is accepted into the backlog at worst.
        pub fn connect(&mut self) -> TcpStream {
            let start = Instant::now();
            let port = loop {
                if let Some(p) = self.port {
                    break p;
                }
                match self.port_rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(p) => {
                        self.port = Some(p);
                        break p;
                    }
                    Err(_) => {
                        if let Ok(Some(status)) = self.child.try_wait() {
                            panic!("openssl s_server exited before listening ({status})");
                        }
                        assert!(start.elapsed() < Duration::from_secs(30), "openssl s_server didn't start within 30 s");
                    }
                }
            };
            let s = TcpStream::connect(("127.0.0.1", port)).unwrap_or_else(|e| panic!("connecting to openssl s_server on port {port}: {e}"));
            s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
            s
        }
    }

    impl Running {
        pub fn type_line(&mut self, line: &str) {
            use std::io::Write;
            let stdin = self.child.stdin.as_mut().unwrap();
            stdin.write_all(format!("{line}\n").as_bytes()).unwrap();
            stdin.flush().unwrap();
        }

        /// Whether the server prints `text` within ten seconds.
        pub fn wait_for_output(&mut self, text: &str) -> bool {
            let rx = self.output.as_ref().unwrap();
            let mut seen = Vec::new();
            let deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < deadline {
                if let Ok(chunk) = rx.recv_timeout(Duration::from_millis(100)) {
                    seen.extend(chunk);
                    if String::from_utf8_lossy(&seen).contains(text) {
                        return true;
                    }
                }
            }
            false
        }
    }

    impl Drop for Running {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// A small seeded generator for the mutation tests (not for keys).
struct Mutator(u64);

impl Mutator {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    /// Flips, overwrites, inserts, deletes or truncates a few bytes.
    fn mutate(&mut self, mut data: Vec<u8>) -> Vec<u8> {
        for _ in 0..1 + self.below(4) {
            let i = self.below(data.len());
            match self.below(5) {
                0 if !data.is_empty() => data[i] ^= 1 << self.below(8),
                1 if !data.is_empty() => data[i] = self.next() as u8,
                2 => data.insert(i, self.next() as u8),
                3 if !data.is_empty() => {
                    data.remove(i);
                }
                _ => data.truncate(i),
            }
        }
        data
    }
}

fn fuzz_count(default: u64) -> u64 {
    std::env::var("AIL_FUZZ_N").ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

// The parsers and the record layer never panic on mutated input
// (ADR 0269's policy, applied to TLS by ADR 0367). `AIL_FUZZ_N` raises
// the count; `./ail selftest` runs a quick pass.
#[test]
fn fuzz_tls_parsers() {
    let n = fuzz_count(3000);
    let mut m = Mutator(0x7115);
    let (client, server) = illustrated();
    let hello = client[5..5 + 248].to_vec();
    let sh_body = server[5 + 4..5 + 122].to_vec();
    let session_id = &hello[39..71];
    let mut cert_body = vec![0u8];
    let mut list = Vec::new();
    for len in [300usize, 20] {
        put_vec(&mut list, 3, &vec![0x30; len]);
        put_vec(&mut list, 2, &[]);
    }
    put_vec(&mut cert_body, 3, &list);
    let verifier = AcceptAny::default();
    let config = Config { host: "example.ulfheim.net", verifier: &verifier };
    for i in 0..n {
        let _ = parse_server_hello(&m.mutate(sh_body.clone()), session_id);
        let _ = parse_certificate(&m.mutate(cert_body.clone()));
        // The whole handshake over a mutated server stream (fewer: each
        // one does an X25519 and some AES-GCM).
        if i % 10 == 0 {
            let input = m.mutate(server.clone());
            let replay = Replay { input, pos: 0, written: Vec::new() };
            if let Ok(mut tls) = handshake_with(replay, &config, illustrated_ephemeral(), Some(hello.clone())) {
                let mut sink = Vec::new();
                let _ = tls.read_to_end(&mut sink);
            }
        }
    }
}
