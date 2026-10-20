//! End-to-end tests of `std.net` (std.net.1-4) against an HTTP server on
//! the loopback interface that the test starts, so they need no network.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;

fn ail(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(args)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .output()
        .unwrap()
}

/// Answer one request: the routes `tests/net/client.ail` asks for.
fn serve(stream: TcpStream) {
    let mut r = BufReader::new(stream.try_clone().unwrap());
    let mut line = String::new();
    r.read_line(&mut line).unwrap();
    let mut words = line.split_whitespace();
    let (method, path) = (words.next().unwrap().to_string(), words.next().unwrap().to_string());
    let (mut length, mut content_type) = (0, String::new());
    loop {
        let mut h = String::new();
        r.read_line(&mut h).unwrap();
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        let (name, value) = h.split_once(':').unwrap();
        match name.to_ascii_lowercase().as_str() {
            "content-length" => length = value.trim().parse().unwrap(),
            "content-type" => content_type = value.trim().to_string(),
            _ => {}
        }
    }
    let mut body = vec![0; length];
    r.read_exact(&mut body).unwrap();
    let mut w = stream;
    let reply = match path.as_str() {
        "/hello" => "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nX-Twice: 1\r\nX-Twice: 2\r\nContent-Length: 12\r\n\r\nhello, agent".to_string(),
        "/moved" => "HTTP/1.1 302 Found\r\nLocation: /hello\r\nContent-Length: 0\r\n\r\n".to_string(),
        "/chunked" => "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n".to_string(),
        "/echo" => {
            let text = format!("{method} {path} {content_type} {length} {}", String::from_utf8_lossy(&body));
            format!("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{text}")
        }
        _ => "HTTP/1.1 404 Not Found\r\nContent-Length: 12\r\n\r\nno such page".to_string(),
    };
    w.write_all(reply.as_bytes()).unwrap();
}

#[test]
fn get_and_post_against_a_local_server() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            serve(stream.unwrap());
        }
    });
    let out = ail(&["run", "bootstrap/tests/net/client.ail", &base]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), include_str!("net/client.out"));
}

#[test]
fn net_is_a_capability() {
    // eff.cap.1, std.net.1: `--deny net` and `--deterministic` refuse it.
    for flag in [["--deny", "net"].as_slice(), ["--deterministic"].as_slice()] {
        let mut args = vec!["run"];
        args.extend_from_slice(flag);
        args.push("bootstrap/tests/net/client.ail");
        let out = ail(&args);
        assert_eq!(out.status.code(), Some(2), "{flag:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(stderr.contains("eff.cap.1") && stderr.contains("`net`"), "{stderr}");
    }
    let out = ail(&["effects", "bootstrap/tests/net/client.ail"]);
    assert!(String::from_utf8_lossy(&out.stdout).contains("net"));
}

/// std.net.5: `https` end to end. `openssl s_server` serves the test
/// chain for `localhost` (bootstrap/src/net/x509/testdata) and the test
/// root is the only trusted one (SSL_CERT_FILE). Skips without OpenSSL
/// 1.1.1 or 3 (macOS's `openssl` is LibreSSL).
#[test]
fn https_against_a_local_tls_server() {
    let version = Command::new("openssl").arg("version").output();
    let Ok(version) = version else {
        eprintln!("note: no `openssl`; skipping the https test");
        return;
    };
    let text = String::from_utf8_lossy(&version.stdout);
    if !(text.starts_with("OpenSSL 3") || text.starts_with("OpenSSL 1.1.1")) {
        eprintln!("note: `openssl` is `{}`; skipping the https test", text.trim());
        return;
    }
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/src/net/x509/testdata");
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let mut server = Command::new("openssl")
        .args(["s_server", "-quiet", "-www", "-tls1_3", "-accept", &port.to_string()])
        .args(["-key", "localhost_leaf.key.pem", "-cert", "localhost_leaf.pem", "-cert_chain", "inter.pem"])
        .current_dir(data)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(start.elapsed().as_secs() < 30, "openssl s_server didn't start");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let run = |roots: &str, urls: &[String]| {
        let mut args = vec!["run".to_string(), "bootstrap/tests/net/https.ail".to_string()];
        args.extend_from_slice(urls);
        let out = Command::new(env!("CARGO_BIN_EXE_ail"))
            .args(&args)
            .env("SSL_CERT_FILE", format!("{data}/{roots}"))
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    // Each request is its own connection; s_server -www serves them in turn.
    let ok = format!("https://localhost:{port}/");
    let by_ip = format!("https://127.0.0.1:{port}/");
    let out = run("root.pem", &[ok.clone(), by_ip]);
    assert_eq!(
        out,
        format!(
            "200 <HTML><BODY BGCOLOR=\"#ffffff\">\n\
             tls: the server's certificate isn't valid for `127.0.0.1`; it covers localhost\n"
        )
    );
    // A root store that doesn't hold the test root.
    let out = run("rsa_leaf.pem", &[ok]);
    assert_eq!(
        out,
        "tls: the certificate for `localhost` doesn't lead to a trusted root: it was issued by `AIL Test Intermediate`, which isn't in the system's root store\n"
    );
    let _ = server.kill();
    let _ = server.wait();
}

/// std.net.13: a raw TLS `Conn` against the same kind of local server.
#[test]
fn tls_conn_against_a_local_tls_server() {
    let version = Command::new("openssl").arg("version").output();
    let Ok(version) = version else {
        eprintln!("note: no `openssl`; skipping the TLS Conn test");
        return;
    };
    let text = String::from_utf8_lossy(&version.stdout);
    if !(text.starts_with("OpenSSL 3") || text.starts_with("OpenSSL 1.1.1")) {
        eprintln!("note: `openssl` is `{}`; skipping the TLS Conn test", text.trim());
        return;
    }
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/src/net/x509/testdata");
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let mut server = Command::new("openssl")
        .args(["s_server", "-quiet", "-www", "-tls1_3", "-accept", &port.to_string()])
        .args(["-key", "localhost_leaf.key.pem", "-cert", "localhost_leaf.pem", "-cert_chain", "inter.pem"])
        .current_dir(data)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(start.elapsed().as_secs() < 30, "openssl s_server didn't start");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let out = Command::new(env!("CARGO_BIN_EXE_ail"))
        .args(["run", "bootstrap/tests/net/tls_conn.ail", &port.to_string()])
        .env("SSL_CERT_FILE", format!("{data}/root.pem"))
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
        .output()
        .unwrap();
    let _ = server.kill();
    let _ = server.wait();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "HTTP/1.0 200 ok\n\
         tls: the server's certificate isn't valid for `127.0.0.1`; it covers localhost\n"
    );
}

/// std.net.14: serving is a capability of its own, on top of `net`.
#[test]
fn serve_is_a_capability() {
    let program = "bootstrap/tests/cases/std_net_serve.ail";
    let out = ail(&["run", "--deny", "net.serve", program]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("eff.cap.1") && stderr.contains("`net.serve`"), "{stderr}");
    // Denying `net` denies serving too, and so does `--deterministic`.
    for flag in ["--deny=net", "--deterministic"] {
        let args: Vec<&str> = if flag == "--deny=net" { vec!["check", "--deny", "net", program] } else { vec!["check", flag, program] };
        assert_eq!(ail(&args).status.code(), Some(2), "{flag}");
    }
    // A client-only program is not denied `net.serve`.
    let out = ail(&["check", "--deny", "net.serve", "bootstrap/tests/net/client.ail"]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let out = ail(&["effects", program]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("fn:main {fail, io, net, net.serve, spawn}"), "{text}");
    assert!(text.contains("fn:worker {fail, io, net}"), "{text}");
}

