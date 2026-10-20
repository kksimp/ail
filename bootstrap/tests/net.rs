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
