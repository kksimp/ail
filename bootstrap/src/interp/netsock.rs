//! Sockets and the HTTP server of `std.net` (spec 17-std, std.net.7-16,
//! ADRs 0418-0423): `net.serve` and its `Server` and `Exchange`, raw TCP
//! with `net.listen`, `net.connect` and `Conn`, and the transport that lets
//! a client and a server in one program wait on each other.
//!
//! Tasks take turns (ADR 0186), so a task that waits on the network must
//! hand the turn on, or the task it waits for never runs. Every socket here
//! is non-blocking. A wait loops: poll, let the other ready tasks run, back
//! off, poll again (std.net.8).

use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use super::builtins::Table;
use super::stdlib::fail;
use super::tasks::Yielder;
use super::{Env, Flow, Interp, Value};
use crate::ast::{Arg, Expr};
use crate::diag::Pos;

/// The largest request head the server reads (std.net.10).
const MAX_HEAD: usize = 64 * 1024;
/// The largest request body the server reads (std.net.10).
const MAX_BODY: usize = 16 * 1024 * 1024;
/// How long a connection may take to send a whole request (std.net.10).
const REQUEST_TIME: Duration = Duration::from_secs(10);

thread_local! {
    /// The running task's way to let other tasks run (std.net.8). Each task
    /// runs on its own thread, so this is per task.
    static YIELD: RefCell<Option<Yielder>> = const { RefCell::new(None) };
}

/// Called at the start of every network operation.
pub(super) fn set_yielder(y: Option<Yielder>) {
    YIELD.with(|c| *c.borrow_mut() = y);
}

/// One step of waiting for the network: let other tasks run, then back off
/// (50 microseconds growing to 2 milliseconds). `Err` if the task was
/// cancelled or the program is shutting down.
fn pause(idle: &mut u32) -> io::Result<()> {
    let gone = YIELD.with(|c| match &*c.borrow() {
        Some(y) => y.yield_now().is_err() || y.cancelled(),
        None => false,
    });
    if gone {
        return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "the task was cancelled"));
    }
    std::thread::sleep(Duration::from_micros(50u64 << (*idle).min(5)));
    *idle += 1;
    Ok(())
}

/// A stream that waits cooperatively (std.net.8): the socket is non-blocking
/// and a read or write that would block pauses and tries again until the
/// inactivity timeout runs out. TLS runs over it unchanged.
pub struct Coop {
    stream: TcpStream,
    timeout: Arc<AtomicU64>,
}

impl Coop {
    pub fn for_client(stream: TcpStream, timeout: Duration) -> io::Result<Coop> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Coop { stream, timeout: Arc::new(AtomicU64::new(timeout.as_millis() as u64)) })
    }

    fn wait<T>(&mut self, mut op: impl FnMut(&mut TcpStream) -> io::Result<T>) -> io::Result<T> {
        let start = Instant::now();
        let mut idle = 0;
        loop {
            match op(&mut self.stream) {
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if start.elapsed() >= Duration::from_millis(self.timeout.load(Ordering::Relaxed)) {
                        return Err(io::ErrorKind::TimedOut.into());
                    }
                    pause(&mut idle)?;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                r => return r,
            }
        }
    }
}

impl Read for Coop {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.wait(|s| s.read(buf))
    }
}

impl Write for Coop {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.wait(|s| s.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

trait Duplex: Read + Write + Send {}
impl<T: Read + Write + Send> Duplex for T {}

/// A raw TCP connection, plain or TLS (std.net.12).
struct Conn {
    io: Box<dyn Duplex>,
    /// Bytes read from the peer and not yet returned.
    buf: Vec<u8>,
    eof: bool,
    timeout: Arc<AtomicU64>,
    peer: String,
}

impl Conn {
    /// Reads once: at least a byte, or end of stream.
    fn fill(&mut self) -> io::Result<()> {
        let mut chunk = [0u8; 16 * 1024];
        let n = self.io.read(&mut chunk)?;
        if n == 0 {
            self.eof = true;
        }
        self.buf.extend_from_slice(&chunk[..n]);
        Ok(())
    }

    /// Up to `max` bytes as text, waiting for at least one; `""` at the end
    /// of the stream. A character is never split across two reads
    /// (std.net.12); bytes that aren't UTF-8 become U+FFFD.
    fn read_text(&mut self, max: usize) -> io::Result<String> {
        if self.buf.is_empty() && !self.eof {
            self.fill()?;
        }
        loop {
            let mut n = max.min(self.buf.len());
            while n < self.buf.len() && self.buf[n] & 0xC0 == 0x80 {
                n += 1;
            }
            match std::str::from_utf8(&self.buf[..n]) {
                Ok(s) => {
                    let s = s.to_string();
                    self.buf.drain(..n);
                    return Ok(s);
                }
                Err(e) if e.error_len().is_none() && n == self.buf.len() && !self.eof => self.fill()?,
                Err(_) => {
                    let s = String::from_utf8_lossy(&self.buf[..n]).into_owned();
                    self.buf.drain(..n);
                    return Ok(s);
                }
            }
        }
    }

    /// Up to `max` bytes, undecoded, waiting for at least one; empty at the
    /// end of the stream (std.net.16).
    fn read_raw(&mut self, max: usize) -> io::Result<Vec<u8>> {
        if self.buf.is_empty() && !self.eof {
            self.fill()?;
        }
        let n = max.min(self.buf.len());
        Ok(self.buf.drain(..n).collect())
    }

    /// A line without its `\n` or `\r\n`, or `None` at the end of the stream.
    fn read_line(&mut self) -> io::Result<Option<String>> {
        let mut from = 0;
        loop {
            if let Some(i) = self.buf[from..].iter().position(|&b| b == b'\n') {
                let mut line: Vec<u8> = self.buf.drain(..=from + i).collect();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }
            from = self.buf.len();
            if self.eof {
                if self.buf.is_empty() {
                    return Ok(None);
                }
                let line = String::from_utf8_lossy(&self.buf).into_owned();
                self.buf.clear();
                return Ok(Some(line));
            }
            self.fill()?;
        }
    }
}

/// A connection that has sent part of a request.
struct Pending {
    stream: TcpStream,
    buf: Vec<u8>,
    since: Instant,
    peer: String,
    head: Option<Head>,
    continued: bool,
}

/// An HTTP server (std.net.9): the listener and the connections still
/// sending their requests.
struct Server {
    listener: TcpListener,
    addr: String,
    pending: Vec<Pending>,
}

/// An accepted request waiting for its response (std.net.11).
struct Exchange {
    stream: TcpStream,
    method: String,
    peer: String,
}

enum Slot {
    Server(Server),
    Listener(TcpListener, String),
    Conn(Conn),
    Exchange(Exchange),
    /// Taken out by an operation that is waiting; `close` while busy
    /// leaves `Gone` so the operation drops it when it ends.
    Busy,
    Gone,
}

static SOCKETS: Mutex<Vec<Slot>> = Mutex::new(Vec::new());

fn sockets() -> MutexGuard<'static, Vec<Slot>> {
    SOCKETS.lock().unwrap_or_else(|e| e.into_inner())
}

fn add(slot: Slot) -> i64 {
    // Closed slots are never reused, so a stale handle can't reach a new socket.
    let mut t = sockets();
    t.push(slot);
    t.len() as i64 - 1
}

/// Takes a connection out for an operation that may wait.
fn take_conn(id: i64) -> Option<Conn> {
    let mut t = sockets();
    let s = t.get_mut(usize::try_from(id).ok()?)?;
    match std::mem::replace(s, Slot::Busy) {
        Slot::Conn(c) => Some(c),
        other => {
            *s = other;
            None
        }
    }
}

/// Takes an exchange out to answer it.
fn take_exchange(id: i64) -> Option<Exchange> {
    let mut t = sockets();
    let s = t.get_mut(usize::try_from(id).ok()?)?;
    match std::mem::replace(s, Slot::Busy) {
        Slot::Exchange(e) => Some(e),
        other => {
            *s = other;
            None
        }
    }
}

fn put_back(id: i64, slot: Slot) {
    let mut t = sockets();
    if let Some(s) = t.get_mut(id as usize)
        && matches!(s, Slot::Busy)
    {
        *s = slot;
    }
    // Otherwise it was closed meanwhile: dropping `slot` closes it.
}

/// Closes a socket. One that an operation has taken out is dropped when
/// that operation puts it back.
fn close(id: i64) {
    let mut t = sockets();
    if let Some(s) = usize::try_from(id).ok().and_then(|i| t.get_mut(i)) {
        *s = Slot::Gone;
    }
}

// ---- HTTP request parsing (std.net.10) ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Framing {
    Length(usize),
    Chunked,
}

#[derive(Clone, Debug)]
struct Head {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body_start: usize,
    framing: Framing,
    expect_continue: bool,
}

/// A request the server refuses: status and reason.
type Bad = (u16, &'static str);

/// A request as the program sees it.
struct Parsed {
    method: String,
    path: String,
    query: Vec<(String, String)>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The request line and headers, once all of them have arrived (RFC 9112).
fn parse_head(buf: &[u8]) -> Result<Option<Head>, Bad> {
    let Some(end) = find(buf, b"\r\n\r\n") else {
        return if buf.len() > MAX_HEAD { Err((431, "the request head is too large")) } else { Ok(None) };
    };
    if end > MAX_HEAD {
        return Err((431, "the request head is too large"));
    }
    let text = std::str::from_utf8(&buf[..end]).map_err(|_| (400, "the request head is not UTF-8"))?;
    let mut lines = text.split("\r\n");
    let line = lines.next().unwrap_or("");
    let mut parts = line.split(' ');
    let (Some(method), Some(target), Some(version), None) = (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err((400, "a request line is `METHOD TARGET HTTP/1.1`"));
    };
    if method.is_empty() || !method.bytes().all(super::net::is_token_byte) {
        return Err((400, "the method is not a token"));
    }
    let http11 = match version {
        "HTTP/1.1" => true,
        "HTTP/1.0" => false,
        _ => return Err((505, "only HTTP/1.0 and HTTP/1.1 are supported")),
    };
    if !(target.starts_with('/') || (target == "*" && method == "OPTIONS")) || target.bytes().any(|b| b < 0x21 || b == 0x7f) {
        return Err((400, "the request target must be an absolute path"));
    }
    let mut headers: Vec<(String, String)> = Vec::new();
    for line in lines {
        if line.starts_with([' ', '\t']) {
            return Err((400, "folded header lines are not allowed"));
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err((400, "a header line is `name: value`"));
        };
        if name.is_empty() || !name.bytes().all(super::net::is_token_byte) {
            return Err((400, "a header name must be a token with no space before the colon"));
        }
        let (name, value) = (name.to_ascii_lowercase(), value.trim_matches([' ', '\t']).to_string());
        if value.bytes().any(|b| b == 0 || (b < 0x20 && b != b'\t')) {
            return Err((400, "a header value has a control character"));
        }
        match headers.iter_mut().find(|(k, _)| *k == name) {
            // A repeated Content-Length is checked below, not joined.
            Some((_, v)) => {
                v.push_str(", ");
                v.push_str(&value);
            }
            None => headers.push((name, value)),
        }
    }
    let header = |n: &str| headers.iter().find(|(k, _)| k == n).map(|(_, v)| v.as_str());
    if http11 && header("host").is_none() {
        return Err((400, "an HTTP/1.1 request needs a Host header"));
    }
    let chunked = match header("transfer-encoding") {
        None => false,
        Some(v) if v.eq_ignore_ascii_case("chunked") => true,
        Some(_) => return Err((501, "only the chunked transfer coding is supported")),
    };
    let framing = match (chunked, header("content-length")) {
        // Both is the request smuggling shape (RFC 9112 6.3).
        (true, Some(_)) => return Err((400, "a request has both Transfer-Encoding and Content-Length")),
        (true, None) => Framing::Chunked,
        (false, Some(v)) => {
            if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
                return Err((400, "Content-Length must be one number"));
            }
            let n: usize = v.parse().map_err(|_| (413, "the request body is too large"))?;
            if n > MAX_BODY {
                return Err((413, "the request body is too large"));
            }
            Framing::Length(n)
        }
        (false, None) => Framing::Length(0),
    };
    let expect_continue = header("expect").is_some_and(|v| v.eq_ignore_ascii_case("100-continue"));
    Ok(Some(Head {
        method: method.to_string(),
        target: target.to_string(),
        headers,
        body_start: end + 4,
        framing,
        expect_continue,
    }))
}

/// The request body, once all of it has arrived.
fn parse_body(head: &Head, buf: &[u8]) -> Result<Option<Vec<u8>>, Bad> {
    let data = &buf[head.body_start..];
    match head.framing {
        Framing::Length(n) => Ok((data.len() >= n).then(|| data[..n].to_vec())),
        Framing::Chunked => {
            let mut body = Vec::new();
            let mut at = 0;
            loop {
                let Some(i) = find(&data[at..], b"\r\n") else { return Ok(None) };
                let line = std::str::from_utf8(&data[at..at + i]).map_err(|_| (400, "a bad chunk size"))?;
                let size_text = line.split(';').next().unwrap_or("").trim();
                let size = usize::from_str_radix(size_text, 16).map_err(|_| (400, "a bad chunk size"))?;
                at += i + 2;
                if size == 0 {
                    // Trailers, up to the blank line.
                    loop {
                        let Some(j) = find(&data[at..], b"\r\n") else { return Ok(None) };
                        at += j + 2;
                        if j == 0 {
                            return Ok(Some(body));
                        }
                    }
                }
                if body.len() + size > MAX_BODY {
                    return Err((413, "the request body is too large"));
                }
                if data.len() < at + size + 2 {
                    return Ok(None);
                }
                body.extend_from_slice(&data[at..at + size]);
                if &data[at + size..at + size + 2] != b"\r\n" {
                    return Err((400, "a chunk is not followed by a line break"));
                }
                at += size + 2;
            }
        }
    }
}

/// Splits `%XX` escapes and `+` in a query component; a bad escape stays as written.
fn decode_component(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() && b[i + 1].is_ascii_hexdigit() && b[i + 2].is_ascii_hexdigit() => {
                out.push(u8::from_str_radix(&s[i + 1..i + 3], 16).unwrap_or(b'%'));
                i += 2;
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `a=1&b=2` as pairs in order; a repeated key keeps its last value when it
/// becomes a map (std.net.10).
fn parse_query(q: &str) -> Vec<(String, String)> {
    q.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (decode_component(k), decode_component(v))
        })
        .collect()
}

fn reason(status: i64) -> &'static str {
    match status {
        100 => "Continue",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        206 => "Partial Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        413 => "Content Too Large",
        415 => "Unsupported Media Type",
        418 => "I'm a Teapot",
        422 => "Unprocessable Content",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        _ => "",
    }
}

/// The bytes of a response (std.net.11). `head_only` omits the body of a
/// reply to `HEAD`. Headers the server owns are not taken from the program.
fn response_bytes(status: i64, headers: &[(String, String)], body: &[u8], head_only: bool) -> Result<Vec<u8>, String> {
    if !(100..=599).contains(&status) {
        return Err(format!("{status} is not an HTTP status; use 100 to 599"));
    }
    let mut out = format!("HTTP/1.1 {status} {}\r\n", reason(status));
    let mut has_type = false;
    for (name, value) in headers {
        if name.is_empty() || !name.bytes().all(super::net::is_token_byte) {
            return Err(format!("`{name}` is not a header name"));
        }
        if value.bytes().any(|b| b == b'\r' || b == b'\n' || b == 0) {
            return Err(format!("the value of header `{name}` has a line break"));
        }
        let lower = name.to_ascii_lowercase();
        if matches!(lower.as_str(), "content-length" | "connection" | "transfer-encoding") {
            continue;
        }
        has_type |= lower == "content-type";
        out.push_str(&format!("{name}: {value}\r\n"));
    }
    let no_body = status / 100 == 1 || status == 204 || status == 304;
    if !has_type && !body.is_empty() && !no_body {
        out.push_str("Content-Type: text/plain; charset=utf-8\r\n");
    }
    if !(status / 100 == 1 || status == 204) {
        out.push_str(&format!("Content-Length: {}\r\n", if no_body { 0 } else { body.len() }));
    }
    out.push_str("Connection: close\r\n\r\n");
    let mut bytes = out.into_bytes();
    if !no_body && !head_only {
        bytes.extend_from_slice(body);
    }
    Ok(bytes)
}

/// Best effort: answer a connection the server is dropping.
fn refuse(mut stream: TcpStream, status: u16, message: &str) {
    let body = format!("{message}\n");
    let text = format!(
        "HTTP/1.1 {status} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        reason(i64::from(status)),
        body.len()
    );
    let _ = stream.write_all(text.as_bytes());
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

enum Poll {
    Ready(Parsed, Exchange),
    Pending,
    Closed,
}

impl Server {
    /// Takes new connections, reads what the pending ones have sent, and
    /// returns the first whole request (std.net.9). Never waits.
    fn poll(&mut self) -> Poll {
        while let Ok((stream, peer)) = self.listener.accept() {
            if stream.set_nonblocking(true).is_ok() {
                let _ = stream.set_nodelay(true);
                self.pending.push(Pending {
                    stream,
                    buf: Vec::new(),
                    since: Instant::now(),
                    peer: peer.to_string(),
                    head: None,
                    continued: false,
                });
            }
        }
        let mut i = 0;
        while i < self.pending.len() {
            let p = &mut self.pending[i];
            let (mut eof, mut grew) = (false, false);
            loop {
                let mut chunk = [0u8; 16 * 1024];
                match p.stream.read(&mut chunk) {
                    Ok(0) => {
                        eof = true;
                        break;
                    }
                    Ok(n) => {
                        p.buf.extend_from_slice(&chunk[..n]);
                        grew = true;
                        if p.buf.len() > MAX_HEAD + MAX_BODY + 1024 {
                            break;
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(_) => {
                        eof = true;
                        break;
                    }
                }
            }
            let outcome: Result<Option<(Head, Vec<u8>)>, Bad> = (|| {
                if p.head.is_none() && (grew || eof) {
                    p.head = parse_head(&p.buf)?;
                }
                let Some(head) = &p.head else { return Ok(None) };
                if head.expect_continue && !p.continued {
                    p.continued = true;
                    let _ = p.stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
                }
                let chunked_ready = head.framing != Framing::Chunked || p.buf.ends_with(b"\r\n\r\n");
                if !(grew || eof) || !chunked_ready {
                    return Ok(None);
                }
                Ok(parse_body(head, &p.buf)?.map(|body| (head.clone(), body)))
            })();
            match outcome {
                Ok(Some((head, body))) => {
                    let p = self.pending.remove(i);
                    let (path, query) = match head.target.split_once('?') {
                        Some((a, b)) => (a.to_string(), parse_query(b)),
                        None => (head.target.clone(), Vec::new()),
                    };
                    let parsed = Parsed { method: head.method.clone(), path, query, headers: head.headers, body };
                    return Poll::Ready(parsed, Exchange { stream: p.stream, method: head.method, peer: p.peer });
                }
                Ok(None) if !eof && p.since.elapsed() < REQUEST_TIME => i += 1,
                Ok(None) => {
                    let p = self.pending.remove(i);
                    if !eof {
                        refuse(p.stream, 408, "the request took too long");
                    }
                }
                Err((status, message)) => {
                    let p = self.pending.remove(i);
                    refuse(p.stream, status, message);
                }
            }
        }
        Poll::Pending
    }
}

fn text(s: impl Into<Rc<str>>) -> Value {
    Value::Str(s.into())
}

/// A map value from pairs; a repeated key keeps its last value.
fn map_of(pairs: Vec<(String, String)>) -> Value {
    let mut t = Table::default();
    for (k, v) in pairs {
        t.insert(text(k), Some(text(v)));
    }
    Value::Map(Rc::new(t))
}

fn pairs_of(v: &Value) -> Vec<(String, String)> {
    let Value::Map(t) = v else { unreachable!("checked: a Map[str, str]") };
    t.entries()
        .map(|(k, v)| match (k, v) {
            (Value::Str(k), Value::Str(v)) => (k.to_string(), v.to_string()),
            _ => unreachable!("checked: str keys and values"),
        })
        .collect()
}

/// The pairs of a `Map[str, List[str]]`: one pair per value (std.net.17).
fn list_pairs(v: &Value) -> Vec<(String, String)> {
    let Value::Map(t) = v else { return Vec::new() };
    let mut out = Vec::new();
    for (k, vs) in t.entries() {
        let (Value::Str(k), Value::List(vs)) = (k, vs) else { continue };
        for v in vs.iter() {
            if let Value::Str(v) = v {
                out.push((k.to_string(), v.to_string()));
            }
        }
    }
    out
}

/// A failure's kind for a socket error (std.net.4).
fn kind_of(e: &io::Error) -> &'static str {
    use io::ErrorKind as K;
    match e.kind() {
        K::ConnectionRefused => "refused",
        K::TimedOut | K::WouldBlock => "timed_out",
        K::InvalidData => "tls",
        K::ConnectionAborted | K::ConnectionReset | K::BrokenPipe | K::UnexpectedEof | K::NotConnected => "closed",
        K::AddrInUse | K::AddrNotAvailable | K::PermissionDenied => "bind",
        _ => "other",
    }
}

fn resolve_addr(addr: &str) -> Result<SocketAddr, (&'static str, String)> {
    let mut it = addr.to_socket_addrs().map_err(|e| ("bad_url", format!("`{addr}` is not an address (HOST:PORT): {e}")))?;
    it.next().ok_or(("not_found", format!("`{addr}` has no address")))
}

impl<'a, W: Write> Interp<'a, W> {
    fn net_error(&self, url: &str, kind: &str, message: impl Into<String>, pos: Pos) -> Value {
        let s = self.structs["net.NetError"];
        fail(Value::Struct(s, Rc::new(vec![text(url), text(kind), text(message.into())])), pos)
    }

    fn handle_struct(&self, name: &str, id: i64) -> Value {
        Value::Struct(self.structs[name], Rc::new(vec![Value::Int(id)]))
    }

    /// `net.serve(addr)` (std.net.9): a `Server` listening on `addr`.
    pub(super) fn net_serve(&mut self, addr: &str, pos: Pos) -> Value {
        match bind(addr) {
            Ok((listener, shown)) => {
                let id = add(Slot::Server(Server { listener, addr: shown, pending: Vec::new() }));
                self.handle_struct("net.Server", id)
            }
            Err((kind, message)) => self.net_error(addr, kind, message, pos),
        }
    }

    /// `net.listen(addr)` (std.net.12): a `Listener` for raw TCP.
    pub(super) fn net_listen(&mut self, addr: &str, pos: Pos) -> Value {
        match bind(addr) {
            Ok((listener, shown)) => {
                let id = add(Slot::Listener(listener, shown));
                self.handle_struct("net.Listener", id)
            }
            Err((kind, message)) => self.net_error(addr, kind, message, pos),
        }
    }

    /// `net.connect(host, port)` and `net.connect_tls(host, port)` (std.net.12).
    pub(super) fn net_connect(&mut self, host: &str, port: i64, tls: bool, pos: Pos) -> Value {
        set_yielder(self.yielder());
        let url = format!("{host}:{port}");
        let Ok(port16) = u16::try_from(port) else {
            return self.net_error(&url, "bad_url", format!("{port} is not a port; use 0 to 65535"), pos);
        };
        let bare = host.trim_start_matches('[').trim_end_matches(']');
        let addrs: Vec<SocketAddr> = match (bare, port16).to_socket_addrs() {
            Ok(a) => a.collect(),
            Err(e) => return self.net_error(&url, "not_found", format!("can't find host `{host}`: {e}"), pos),
        };
        let timeout = super::net::TIMEOUT;
        let mut last = None;
        let mut stream = None;
        for a in &addrs {
            match TcpStream::connect_timeout(a, timeout) {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(e) => last = Some(e),
            }
        }
        let Some(stream) = stream else {
            return match last {
                Some(e) => self.net_error(&url, kind_of(&e), format!("can't connect to {url}: {e}"), pos),
                None => self.net_error(&url, "not_found", format!("host `{host}` has no address"), pos),
            };
        };
        let peer = stream.peer_addr().map_or(url.clone(), |a| a.to_string());
        let coop = match Coop::for_client(stream, timeout) {
            Ok(c) => c,
            Err(e) => return self.net_error(&url, kind_of(&e), e.to_string(), pos),
        };
        let timeout = Arc::clone(&coop.timeout);
        let io: Box<dyn Duplex> = if tls {
            let roots = match crate::net::x509::roots::system() {
                Ok(r) => r,
                Err(e) => return self.net_error(&url, "tls", e, pos),
            };
            let verifier = crate::net::x509::verify::WebVerifier { roots: &roots.certs, now: None };
            let config = crate::net::tls::Config { host: bare, verifier: &verifier };
            match crate::net::tls::connect(coop, &config) {
                Ok(s) => Box::new(s),
                Err(e) => {
                    use crate::net::tls::Kind;
                    let kind = match e.kind {
                        Kind::Io(k) => kind_of(&io::Error::from(k)),
                        Kind::Protocol | Kind::Certificate => "tls",
                    };
                    return self.net_error(&url, kind, e.message, pos);
                }
            }
        } else {
            Box::new(coop)
        };
        let id = add(Slot::Conn(Conn { io, buf: Vec::new(), eof: false, timeout, peer }));
        self.handle_struct("net.Conn", id)
    }

    /// The methods of `Server`, `Exchange`, `Listener` and `Conn` (std.net.9-12).
    pub(super) fn net_method(&mut self, m: &str, recv: &'a Expr, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let Value::Struct(_, fields) = self.eval(recv, env)? else { unreachable!("checked: a net handle") };
        // An `Exchange` holds its request first, then its handle.
        let id = fields[fields.len() - 1].int();
        let mut values = Vec::with_capacity(args.len());
        for a in args {
            values.push(self.eval(&a.value, env)?);
        }
        set_yielder(self.yielder());
        let closed = |me: &Self, what: &str| me.net_error("", "closed", format!("this {what} is closed"), pos);
        Ok(match m {
            "Server.addr" | "Listener.addr" => match sockets().get(id as usize) {
                Some(Slot::Server(s)) => text(s.addr.clone()),
                Some(Slot::Listener(_, a)) => text(a.clone()),
                _ => text(""),
            },
            "Server.close" | "Listener.close" | "Conn.close" => {
                close(id);
                Value::Unit
            }
            "Server.accept" => {
                self.flush(pos)?;
                let mut idle = 0;
                loop {
                    let polled = match sockets().get_mut(id as usize) {
                        Some(Slot::Server(s)) => s.poll(),
                        _ => Poll::Closed,
                    };
                    match polled {
                        Poll::Ready(req, ex) => {
                            let peer = ex.peer.clone();
                            let slot = add(Slot::Exchange(ex));
                            let request = Value::Struct(
                                self.structs["net.Request"],
                                Rc::new(vec![
                                    text(req.method),
                                    text(req.path),
                                    map_of(req.query),
                                    map_of(req.headers),
                                    text(String::from_utf8_lossy(&req.body).into_owned()),
                                    text(peer),
                                    super::bytes::bytes_value(req.body.clone()),
                                ]),
                            );
                            return Ok(Value::Struct(self.structs["net.Exchange"], Rc::new(vec![request, Value::Int(slot)])));
                        }
                        Poll::Closed => return Ok(closed(self, "server")),
                        Poll::Pending => {
                            if pause(&mut idle).is_err() {
                                return Ok(self.net_error("", "closed", "the task was cancelled", pos));
                            }
                        }
                    }
                }
            }
            "Listener.accept" => {
                self.flush(pos)?;
                let mut idle = 0;
                loop {
                    let accepted = match sockets().get_mut(id as usize) {
                        Some(Slot::Listener(l, _)) => Some(l.accept()),
                        _ => None,
                    };
                    match accepted {
                        None => return Ok(closed(self, "listener")),
                        Some(Ok((stream, peer))) => {
                            let coop = match Coop::for_client(stream, super::net::TIMEOUT) {
                                Ok(c) => c,
                                Err(e) => return Ok(self.net_error("", kind_of(&e), e.to_string(), pos)),
                            };
                            let timeout = Arc::clone(&coop.timeout);
                            let conn = Conn { io: Box::new(coop), buf: Vec::new(), eof: false, timeout, peer: peer.to_string() };
                            return Ok(self.handle_struct("net.Conn", add(Slot::Conn(conn))));
                        }
                        Some(Err(e)) if e.kind() == io::ErrorKind::WouldBlock => {
                            if pause(&mut idle).is_err() {
                                return Ok(self.net_error("", "closed", "the task was cancelled", pos));
                            }
                        }
                        Some(Err(e)) if e.kind() == io::ErrorKind::Interrupted => {}
                        Some(Err(e)) => return Ok(self.net_error("", kind_of(&e), e.to_string(), pos)),
                    }
                }
            }
            "Exchange.respond" | "Exchange.respond_bytes" => {
                let Some(ex) = take_exchange(id) else {
                    return Ok(self.net_error("", "other", "this exchange has already been answered or is closed", pos));
                };
                // std.net.16: `respond_bytes(status, headers, data)` sends bytes as they are.
                let (status, headers, body): (i64, Vec<(String, String)>, Vec<u8>) = if m == "Exchange.respond_bytes" {
                    let Value::Bytes(data) = &values[2] else { unreachable!("checked: bytes") };
                    (values[0].int(), pairs_of(&values[1]), data.to_vec())
                } else {
                    let Value::Struct(_, r) = &values[0] else { unreachable!("checked: net.Response") };
                    let Value::Str(body) = &r[2] else { unreachable!("checked: str") };
                    // std.net.17: names in `all_headers` are sent value by value, in place of `headers`'.
                    let many = list_pairs(&r[3]);
                    let mut h: Vec<(String, String)> = pairs_of(&r[1])
                        .into_iter()
                        .filter(|(k, _)| !many.iter().any(|(m, _)| m.eq_ignore_ascii_case(k)))
                        .collect();
                    h.extend(many);
                    (r[0].int(), h, body.as_bytes().to_vec())
                };
                let sent = response_bytes(status, &headers, &body, ex.method == "HEAD");
                let result = match sent {
                    Err(message) => {
                        // The exchange stays open so the program can answer again.
                        put_back(id, Slot::Exchange(ex));
                        return Ok(self.net_error("", "bad_request", message, pos));
                    }
                    Ok(bytes) => {
                        let peer = ex.peer.clone();
                        match Coop::for_client(ex.stream, super::net::TIMEOUT) {
                            Ok(mut c) => {
                                let r = c.write_all(&bytes);
                                let _ = c.stream.shutdown(std::net::Shutdown::Write);
                                r.map_err(|e| (peer, e))
                            }
                            Err(e) => Err((peer, e)),
                        }
                    }
                };
                close(id);
                match result {
                    Ok(()) => Value::Unit,
                    Err((peer, e)) => self.net_error(&peer, kind_of(&e), e.to_string(), pos),
                }
            }
            "Conn.peer" => match sockets().get(id as usize) {
                Some(Slot::Conn(c)) => text(c.peer.clone()),
                _ => text(""),
            },
            "Conn.set_timeout" => {
                let ms = values[0].int();
                if ms <= 0 {
                    return Ok(self.net_error("", "other", format!("a timeout is at least 1 millisecond, not {ms}"), pos));
                }
                if let Some(Slot::Conn(c)) = sockets().get(id as usize) {
                    c.timeout.store(ms as u64, Ordering::Relaxed);
                }
                Value::Unit
            }
            "Conn.read" | "Conn.read_bytes" | "Conn.read_line" | "Conn.write" | "Conn.write_bytes" => {
                self.flush(pos)?;
                let Some(mut c) = take_conn(id) else {
                    return Ok(closed(self, "connection"));
                };
                let peer = c.peer.clone();
                let result = match m {
                    "Conn.read" | "Conn.read_bytes" => {
                        let max = values[0].int();
                        if max <= 0 {
                            put_back(id, Slot::Conn(c));
                            return Ok(self.net_error(&peer, "other", format!("`read` needs a size of at least 1, not {max}"), pos));
                        }
                        if m == "Conn.read_bytes" {
                            c.read_raw(max as usize).map(super::bytes::bytes_value)
                        } else {
                            c.read_text(max as usize).map(text)
                        }
                    }
                    "Conn.read_line" => c.read_line().map(|l| l.map_or(Value::None, text)),
                    _ => {
                        let data: &[u8] = match &values[0] {
                            Value::Str(s) => s.as_bytes(),
                            Value::Bytes(b) => b,
                            other => unreachable!("checked: str or bytes, found {other:?}"),
                        };
                        c.io.write_all(data).map(|()| Value::Unit)
                    }
                };
                put_back(id, Slot::Conn(c));
                match result {
                    Ok(v) => v,
                    Err(e) => self.net_error(&peer, kind_of(&e), e.to_string(), pos),
                }
            }
            _ => unreachable!("checked: `net.{m}`"),
        })
    }
}

fn bind(addr: &str) -> Result<(TcpListener, String), (&'static str, String)> {
    let sa = resolve_addr(addr)?;
    let listener = TcpListener::bind(sa).map_err(|e| (kind_of(&e), format!("can't listen on {addr}: {e}")))?;
    listener.set_nonblocking(true).map_err(|e| ("other", e.to_string()))?;
    let shown = listener.local_addr().map_or_else(|_| addr.to_string(), |a| a.to_string());
    Ok((listener, shown))
}

/// The pairs of a `net.Options` map field, for the client (std.net.6).
pub(super) fn options_pairs(v: &Value) -> Vec<(String, String)> {
    pairs_of(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(s: &str) -> Result<Option<Head>, Bad> {
        parse_head(s.as_bytes())
    }

    #[test]
    fn heads() {
        let h = head("GET /a?x=1 HTTP/1.1\r\nHost: h\r\nX-A: 1\r\nx-a: 2\r\n\r\n").unwrap().unwrap();
        assert_eq!((h.method.as_str(), h.target.as_str()), ("GET", "/a?x=1"));
        assert_eq!(h.headers[1], ("x-a".to_string(), "1, 2".to_string()));
        assert_eq!(h.framing, Framing::Length(0));
        assert!(head("GET / HTTP/1.1\r\nHost: h\r\n").unwrap().is_none());
        assert_eq!(head("GET / HTTP/1.1\r\n\r\n").unwrap_err().0, 400);
        assert_eq!(head("GET / HTTP/2\r\nHost: h\r\n\r\n").unwrap_err().0, 505);
        assert_eq!(head("GET x HTTP/1.1\r\nHost: h\r\n\r\n").unwrap_err().0, 400);
        assert_eq!(head("GET / HTTP/1.1\r\nHost : h\r\n\r\n").unwrap_err().0, 400);
        assert_eq!(head("POST / HTTP/1.1\r\nHost: h\r\nContent-Length: 1x\r\n\r\n").unwrap_err().0, 400);
        assert_eq!(head("POST / HTTP/1.1\r\nHost: h\r\nContent-Length: 1, 1\r\n\r\n").unwrap_err().0, 400);
        assert_eq!(head("POST / HTTP/1.1\r\nHost: h\r\nContent-Length: 99999999999\r\n\r\n").unwrap_err().0, 413);
        let smuggle = "POST / HTTP/1.1\r\nHost: h\r\nContent-Length: 1\r\nTransfer-Encoding: chunked\r\n\r\n";
        assert_eq!(head(smuggle).unwrap_err().0, 400);
        assert_eq!(head("POST / HTTP/1.1\r\nHost: h\r\nTransfer-Encoding: gzip\r\n\r\n").unwrap_err().0, 501);
        assert!(head("GET / HTTP/1.0\r\n\r\n").unwrap().is_some());
        let big = format!("GET / HTTP/1.1\r\nHost: h\r\nX: {}\r\n\r\n", "a".repeat(MAX_HEAD));
        assert_eq!(head(&big).unwrap_err().0, 431);
    }

    #[test]
    fn bodies() {
        let raw = b"POST / HTTP/1.1\r\nHost: h\r\nContent-Length: 5\r\n\r\nhel";
        let h = parse_head(raw).unwrap().unwrap();
        assert!(parse_body(&h, raw).unwrap().is_none());
        let raw = b"POST / HTTP/1.1\r\nHost: h\r\nContent-Length: 5\r\n\r\nhello";
        assert_eq!(parse_body(&h, raw).unwrap().unwrap(), b"hello");
        let raw = b"POST / HTTP/1.1\r\nHost: h\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5;x=1\r\npedia\r\n0\r\nT: 1\r\n\r\n";
        let h = parse_head(raw).unwrap().unwrap();
        assert_eq!(parse_body(&h, raw).unwrap().unwrap(), b"Wikipedia");
        assert!(parse_body(&h, &raw[..raw.len() - 4]).unwrap().is_none());
        let bad = b"POST / HTTP/1.1\r\nHost: h\r\nTransfer-Encoding: chunked\r\n\r\nzz\r\n";
        let h = parse_head(bad).unwrap().unwrap();
        assert_eq!(parse_body(&h, bad).unwrap_err().0, 400);
    }

    #[test]
    fn queries() {
        assert_eq!(decode_component("a%20b+c%zz%4"), "a b c%zz%4");
        assert_eq!(
            parse_query("a=1&b=x%26y&c&=z"),
            vec![
                ("a".into(), "1".into()),
                ("b".into(), "x&y".into()),
                ("c".into(), "".into()),
                ("".into(), "z".into())
            ]
        );
    }

    #[test]
    fn responses() {
        let r = response_bytes(200, &[("X-A".into(), "1".into())], b"hi", false).unwrap();
        let t = String::from_utf8(r).unwrap();
        assert!(t.starts_with("HTTP/1.1 200 OK\r\nX-A: 1\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 2\r\n"));
        assert!(t.ends_with("\r\n\r\nhi"));
        let t = String::from_utf8(response_bytes(204, &[], b"ignored", false).unwrap()).unwrap();
        assert!(!t.contains("Content-Length") && t.ends_with("\r\n\r\n"));
        let t = String::from_utf8(response_bytes(200, &[], b"hi", true).unwrap()).unwrap();
        assert!(t.contains("Content-Length: 2") && t.ends_with("\r\n\r\n"));
        assert!(response_bytes(99, &[], b"", false).is_err());
        assert!(response_bytes(200, &[("X".into(), "a\r\nB: c".into())], b"", false).is_err());
        assert!(response_bytes(200, &[("X Y".into(), "a".into())], b"", false).is_err());
        // The server owns the framing headers.
        let t = String::from_utf8(response_bytes(200, &[("Content-Length".into(), "9".into())], b"hi", false).unwrap()).unwrap();
        assert_eq!(t.matches("Content-Length").count(), 1);
    }

    /// A property test (ADR 0269 style, fixed seed): the parsers never
    /// panic on arbitrary bytes, and a component survives encode then decode.
    #[test]
    fn arbitrary_input() {
        let mut x: u64 = 0x2545_f491_4f6c_dd1d;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let alphabet = b"GET POST/HTTP/1.1\r\n: ;=&%+0123456789abcdefABCDEF-chunked\0\xff";
        for _ in 0..3000 {
            let len = (next() % 200) as usize;
            let raw: Vec<u8> = (0..len).map(|_| alphabet[(next() % alphabet.len() as u64) as usize]).collect();
            if let Ok(Some(h)) = parse_head(&raw) {
                let _ = parse_body(&h, &raw);
            }
            let text = String::from_utf8_lossy(&raw).into_owned();
            let _ = parse_query(&text);
            assert_eq!(decode_component(&crate::interp::net::encode_component(&text)), text);
        }
    }
}
