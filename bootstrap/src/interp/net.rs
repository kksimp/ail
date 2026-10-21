//! `std.net` (spec 17-std, ADR 0283): a minimal HTTP/1.1 client on the
//! standard library's TCP, with `https` over the in-house TLS 1.3 client
//! (std.net.5, ADRs 0360, 0369). Request options and the cooperative
//! transport are std.net.6-8 (ADRs 0418-0420).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::netsock::{Coop, Limits};

/// How long connecting, and each read or write, may take unless the request
/// says otherwise (std.net.2, std.net.6).
pub const TIMEOUT: Duration = Duration::from_secs(30);

/// How many redirects a request follows unless it says otherwise (std.net.2, std.net.6).
pub const MAX_REDIRECTS: usize = 5;

/// What a request can set beyond its method and URL (std.net.6).
#[derive(Clone)]
pub struct Opts {
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub timeout: Duration,
    pub max_redirects: usize,
    /// Connecting alone may take this long; `None` uses `timeout` (std.net.19).
    pub connect: Option<Duration>,
    /// The whole request, redirects included, may take this long (std.net.19).
    pub total: Option<Duration>,
}

impl Default for Opts {
    fn default() -> Opts {
        Opts {
            headers: Vec::new(),
            query: Vec::new(),
            body: Vec::new(),
            content_type: None,
            timeout: TIMEOUT,
            max_redirects: MAX_REDIRECTS,
            connect: None,
            total: None,
        }
    }
}

/// A response: status, headers (names in lowercase, in arrival order, a
/// repeated header's values joined with `, `), and the body as bytes.
pub struct Response {
    pub status: i64,
    pub headers: Vec<(String, String)>,
    /// Each header's values one by one, in arrival order (std.net.17).
    pub all: Vec<(String, Vec<String>)>,
    /// The body's bytes, undecoded (type.bytes.1).
    pub body: Vec<u8>,
}

/// A failure (std.net.4): the URL, its kind and a sentence for people.
pub struct NetFail {
    pub url: String,
    pub kind: &'static str,
    pub message: String,
}

fn fail(url: &str, kind: &'static str, message: impl Into<String>) -> NetFail {
    NetFail { url: url.to_string(), kind, message: message.into() }
}

/// std.net.2: `GET url`, following up to five redirects.
pub fn get(url: &str) -> Result<Response, NetFail> {
    request("GET", url, &Opts::default())
}

/// std.net.3: `POST url` with `body` of type `content_type`. Redirects are
/// returned, not followed.
pub fn post(url: &str, body: &[u8], content_type: &str) -> Result<Response, NetFail> {
    let opts = Opts { body: body.to_vec(), content_type: Some(content_type.to_string()), ..Opts::default() };
    request("POST", url, &opts)
}

/// std.net.6: a request with a method, headers, a query and limits. A
/// redirect of a `GET` or `HEAD` is followed up to `max_redirects` times (0
/// returns the redirect itself); the
/// credentials in `authorization` and `cookie` go only to the host they were
/// set for. Any other method gets its redirect back as the response.
pub fn request(method: &str, url: &str, opts: &Opts) -> Result<Response, NetFail> {
    request_impl(method, url, opts, false).map(|r| r.resp)
}

/// What a request gave: the response, and with `stream` the body not yet read.
pub struct Reply {
    pub resp: Response,
    pub open: Option<Streaming>,
}

/// std.net.25: like `request`, but the body is left to be read in pieces
/// from the returned `Streaming`.
pub fn open(method: &str, url: &str, opts: &Opts) -> Result<(Response, Streaming), NetFail> {
    let r = request_impl(method, url, opts, true)?;
    Ok((r.resp, r.open.expect("a streamed reply has its body")))
}

fn request_impl(method: &str, url: &str, opts: &Opts, stream: bool) -> Result<Reply, NetFail> {
    let method = method.to_ascii_uppercase();
    if method.is_empty() || !method.bytes().all(is_token_byte) {
        return Err(fail(url, "bad_request", format!("`{method}` is not an HTTP method")));
    }
    for (name, value) in &opts.headers {
        check_header(url, name, value)?;
    }
    let mut url = with_query(url, &opts.query);
    let mut opts = opts.clone();
    // std.net.26: unless the program asks for an encoding itself, the client
    // asks for gzip or deflate and undoes it.
    let auto = !stream && !opts.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("accept-encoding"));
    if auto {
        opts.headers.push(("Accept-Encoding".to_string(), "gzip, deflate".to_string()));
    }
    let follows = method == "GET" || method == "HEAD";
    let deadline = opts.total.map(|t| Instant::now() + t);
    for _ in 0..=opts.max_redirects {
        let mut reply = exchange_once(&method, &url, &opts, deadline, stream)?;
        let r = &reply.resp;
        let location = r.headers.iter().find(|(k, _)| k == "location").map(|(_, v)| v.clone());
        match (r.status, location) {
            (301 | 302 | 303 | 307 | 308, Some(to)) if follows && opts.max_redirects > 0 => {
                // The redirect's own body is read out so its connection can be reused.
                if let Some(mut st) = reply.open.take() {
                    st.read_all().map_err(|(kind, message)| fail(&url, kind, message))?;
                }
                let next = resolve(&url, &to);
                if origin(&next) != origin(&url) {
                    opts.headers.retain(|(k, _)| !k.eq_ignore_ascii_case("authorization") && !k.eq_ignore_ascii_case("cookie"));
                }
                url = next;
            }
            _ => {
                if auto {
                    decode_body(&url, &mut reply.resp)?;
                }
                return Ok(reply);
            }
        }
    }
    Err(fail(&url, "protocol", format!("more than {} redirects", opts.max_redirects)))
}

/// Largest body a `Content-Encoding` is undone to (std.net.26).
const DECODED_LIMIT: usize = 256 * 1024 * 1024;

/// std.net.26: undoes `Content-Encoding: gzip` or `deflate`, then drops that
/// header and `Content-Length`, which described the encoded bytes.
fn decode_body(url: &str, r: &mut Response) -> Result<(), NetFail> {
    let enc = r.headers.iter().find(|(k, _)| k == "content-encoding").map(|(_, v)| v.trim().to_ascii_lowercase());
    let Some(enc) = enc else { return Ok(()) };
    if r.body.is_empty() || !matches!(enc.as_str(), "gzip" | "x-gzip" | "deflate") {
        return Ok(());
    }
    let bad = |e: crate::codec::CodecError| fail(url, "protocol", format!("a body that says it is {enc} is not: {}", e.message));
    r.body = if enc == "deflate" {
        // Servers send both the zlib form the standard names and raw deflate.
        crate::codec::zlib::unzlib(&r.body, DECODED_LIMIT).or_else(|_| crate::codec::inflate::inflate(&r.body, DECODED_LIMIT)).map_err(bad)?
    } else {
        crate::codec::gzip::gunzip(&r.body, DECODED_LIMIT).map_err(bad)?
    };
    for gone in ["content-encoding", "content-length"] {
        r.headers.retain(|(k, _)| k != gone);
        r.all.retain(|(k, _)| k != gone);
    }
    Ok(())
}

/// The scheme and authority of a URL, lowercase, for comparing hosts.
fn origin(url: &str) -> String {
    let (scheme, rest) = url.split_once("://").unwrap_or(("", url));
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    format!("{}://{}", scheme.to_ascii_lowercase(), authority.rsplit('@').next().unwrap_or("").to_ascii_lowercase())
}

/// RFC 9110 `tchar`: what a method or header name may contain.
pub fn is_token_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b)
}

/// A header name must be a token and a value must not break the line
/// (std.net.6): the guard against header injection.
fn check_header(url: &str, name: &str, value: &str) -> Result<(), NetFail> {
    if name.is_empty() || !name.bytes().all(is_token_byte) {
        return Err(fail(url, "bad_request", format!("`{name}` is not a header name")));
    }
    if value.bytes().any(|b| b == b'\r' || b == b'\n' || b == 0) {
        return Err(fail(url, "bad_request", format!("the value of header `{name}` has a line break")));
    }
    Ok(())
}

/// Percent-encodes `s` for a query string (RFC 3986 unreserved bytes stay).
pub fn encode_component(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `url` with `query` appended, before any fragment.
fn with_query(url: &str, query: &[(String, String)]) -> String {
    if query.is_empty() {
        return url.to_string();
    }
    let (base, fragment) = match url.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (url, None),
    };
    let pairs: Vec<String> =
        query.iter().map(|(k, v)| format!("{}={}", encode_component(k), encode_component(v))).collect();
    let sep = if base.contains('?') { '&' } else { '?' };
    let mut out = format!("{base}{sep}{}", pairs.join("&"));
    if let Some(f) = fragment {
        out.push('#');
        out.push_str(f);
    }
    out
}

/// A URL's host, port, request target, and whether it's `https`.
struct Target {
    host: String,
    port: u16,
    path: String,
    tls: bool,
}

fn parse_url(url: &str) -> Result<Target, NetFail> {
    let Some((scheme, rest)) = url.split_once("://") else {
        return Err(fail(url, "bad_url", "a URL starts with `http://` or `https://`"));
    };
    let tls = match scheme.to_ascii_lowercase().as_str() {
        "http" => false,
        "https" => true,
        other => {
            return Err(fail(url, "bad_url", format!("`{other}` URLs aren't supported; use `http://` or `https://`")));
        }
    };
    let (authority, path) = match rest.find(['/', '?', '#']) {
        Some(i) if rest.as_bytes()[i] == b'/' => (&rest[..i], &rest[i..]),
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    // The fragment is never sent.
    let path = path.split('#').next().unwrap_or("");
    let path = if path.starts_with('/') { path.to_string() } else { format!("/{path}") };
    let authority = authority.rsplit('@').next().unwrap_or("");
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) if !h.ends_with(':') && !p.contains(']') => {
            let port = p.parse::<u16>().map_err(|_| fail(url, "bad_url", format!("`{p}` is not a port")))?;
            (h, port)
        }
        _ => (authority, if tls { 443 } else { 80 }),
    };
    if host.is_empty() {
        return Err(fail(url, "bad_url", "the URL has no host"));
    }
    Ok(Target { host: host.to_string(), port, path, tls })
}

/// A redirect's `Location`, made absolute against the URL that gave it.
fn resolve(base: &str, location: &str) -> String {
    if location.contains("://") {
        return location.to_string();
    }
    let (scheme, rest) = base.split_once("://").unwrap_or(("http", base));
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if location.starts_with('/') {
        format!("{scheme}://{authority}{location}")
    } else {
        let path = &rest[authority.len()..];
        let dir = path.rfind('/').map_or("/", |i| &path[..=i]);
        format!("{scheme}://{authority}{dir}{location}")
    }
}

/// A client stream, plain or TLS.
trait Duplex: Read + Write + Send {}
impl<T: Read + Write + Send> Duplex for T {}

type Stream = Box<dyn Duplex>;

/// An open connection waiting in the pool (std.net.19).
struct Idle {
    key: (bool, String, u16),
    stream: Stream,
    limits: Limits,
    /// Asks the socket whether the server has closed it meanwhile.
    probe: TcpStream,
    since: Instant,
    /// How long the server said it keeps an idle connection (`Keep-Alive: timeout=`).
    keep: Duration,
}

/// Idle connections kept per host and in all (std.net.19).
const POOL_PER_HOST: usize = 4;
const POOL_TOTAL: usize = 32;
/// How long an idle connection is kept unless the server says less.
const POOL_IDLE: Duration = Duration::from_secs(30);

static POOL: Mutex<Vec<Idle>> = Mutex::new(Vec::new());
/// Connections made and requests that reused one, for `net.pool_stats` and tests.
static OPENED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static REUSED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// `(connections opened, requests served on a reused connection)` so far.
#[cfg(test)]
pub fn pool_stats() -> (u64, u64) {
    use std::sync::atomic::Ordering::Relaxed;
    (OPENED.load(Relaxed), REUSED.load(Relaxed))
}

/// Closes every idle connection (std.net.19).
#[cfg(test)]
pub fn pool_clear() {
    POOL.lock().unwrap_or_else(|e| e.into_inner()).clear();
}

fn still_open(probe: &TcpStream) -> bool {
    let mut one = [0u8; 1];
    if probe.set_nonblocking(true).is_err() {
        return false;
    }
    // Nothing to read means the server is quiet and the connection is open;
    // end of file or stray bytes (a TLS close alert) mean it is not usable.
    matches!(probe.peek(&mut one), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock)
}

fn take_idle(key: &(bool, String, u16)) -> Option<Idle> {
    let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    pool.retain(|i| i.since.elapsed() < i.keep);
    while let Some(at) = pool.iter().rposition(|i| i.key == *key) {
        let idle = pool.remove(at);
        if still_open(&idle.probe) {
            return Some(idle);
        }
    }
    None
}

fn give_back(idle: Idle) {
    let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    while pool.iter().filter(|i| i.key == idle.key).count() >= POOL_PER_HOST || pool.len() >= POOL_TOTAL {
        match pool.iter().position(|i| i.key == idle.key).or(Some(0)) {
            Some(at) if !pool.is_empty() => {
                pool.remove(at);
            }
            _ => break,
        }
    }
    pool.push(idle);
}

fn exchange_once(method: &str, url: &str, opts: &Opts, deadline: Option<Instant>, stream: bool) -> Result<Reply, NetFail> {
    let t = parse_url(url)?;
    let key = (t.tls, t.host.to_ascii_lowercase(), t.port);
    let given = |name: &str| opts.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(name));
    let default_port = if t.tls { 443 } else { 80 };
    let host_header = if t.port == default_port { t.host.clone() } else { format!("{}:{}", t.host, t.port) };
    let chunked = opts.headers.iter().any(|(k, v)| {
        k.eq_ignore_ascii_case("transfer-encoding") && v.to_ascii_lowercase().contains("chunked")
    });
    let closes = opts.headers.iter().any(|(k, v)| k.eq_ignore_ascii_case("connection") && v.eq_ignore_ascii_case("close"));
    let has_body = !opts.body.is_empty() || matches!(method, "POST" | "PUT" | "PATCH");
    let mut head = format!("{method} {} HTTP/1.1\r\n", t.path);
    if !given("host") {
        head.push_str(&format!("Host: {host_header}\r\n"));
    }
    if !given("user-agent") {
        head.push_str("User-Agent: ail\r\n");
    }
    if !given("accept") {
        head.push_str("Accept: */*\r\n");
    }
    if closes {
        head.push_str("Connection: close\r\n");
    }
    for (name, value) in &opts.headers {
        let owned = ["connection", "content-length", "transfer-encoding"];
        if !owned.iter().any(|o| name.eq_ignore_ascii_case(o)) {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
    }
    if let Some(ct) = &opts.content_type
        && !given("content-type")
    {
        head.push_str(&format!("Content-Type: {ct}\r\n"));
    }
    if has_body && chunked {
        // std.net.19: the body goes out as chunks of at most 8 KiB.
        head.push_str("Transfer-Encoding: chunked\r\n");
    } else if has_body {
        head.push_str(&format!("Content-Length: {}\r\n", opts.body.len()));
    }
    head.push_str("\r\n");
    let mut bytes = head.into_bytes();
    if has_body && chunked {
        for part in opts.body.chunks(8192) {
            bytes.extend_from_slice(format!("{:x}\r\n", part.len()).as_bytes());
            bytes.extend_from_slice(part);
            bytes.extend_from_slice(b"\r\n");
        }
        bytes.extend_from_slice(b"0\r\n\r\n");
    } else {
        bytes.extend_from_slice(&opts.body);
    }
    let idempotent = matches!(method, "GET" | "HEAD" | "PUT" | "DELETE" | "OPTIONS");
    // A pooled connection can have been closed by the server in the instant
    // since the check: such a request is sent again on a fresh connection,
    // but only when repeating it is safe (RFC 9110 9.2.2).
    if !closes
        && let Some(idle) = take_idle(&key)
    {
        idle.limits.set(opts.timeout, deadline);
        match run(idle, &bytes, method, url, &key, !closes, stream) {
            Ok(r) => {
                REUSED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Ok(r);
            }
            Err((_, true)) if idempotent => {}
            Err((e, _)) => return Err(e),
        }
    }
    let idle = connect(url, &t, key.clone(), opts, deadline)?;
    run(idle, &bytes, method, url, &key, !closes, stream).map_err(|(e, _)| e)
}

/// Opens a connection to `t` (std.net.5, std.net.19).
fn connect(url: &str, t: &Target, key: (bool, String, u16), opts: &Opts, deadline: Option<Instant>) -> Result<Idle, NetFail> {
    let io = |kind: &'static str, e: std::io::Error| fail(url, kind, e.to_string());
    let host = t.host.trim_start_matches('[').trim_end_matches(']');
    let addrs: Vec<_> = (host, t.port)
        .to_socket_addrs()
        .map_err(|e| fail(url, "not_found", format!("can't find host `{}`: {e}", t.host)))?
        .collect();
    let mut connect_for = opts.connect.unwrap_or(opts.timeout);
    if let Some(d) = deadline {
        connect_for = connect_for.min(d.saturating_duration_since(Instant::now()));
        if connect_for.is_zero() {
            return Err(fail(url, "timed_out", "the request's total time ran out"));
        }
    }
    let mut last = None;
    let mut stream = None;
    for a in &addrs {
        match TcpStream::connect_timeout(a, connect_for) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) => last = Some(e),
        }
    }
    let Some(stream) = stream else {
        return Err(match last {
            Some(e) => fail(url, connect_kind(&e), format!("can't connect to {}:{}: {e}", t.host, t.port)),
            None => fail(url, "not_found", format!("host `{}` has no address", t.host)),
        });
    };
    OPENED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    // The stream waits cooperatively when tasks are running, so a client and
    // a server in one program both make progress (std.net.8).
    let coop = Coop::for_client(stream, opts.timeout).map_err(|e| io("other", e))?;
    let limits = coop.limits();
    limits.set(opts.timeout, deadline);
    let probe = coop.probe().map_err(|e| io("other", e))?;
    let stream: Stream = if !t.tls {
        Box::new(coop)
    } else {
        // std.net.5: TLS 1.3, the certificate checked against the platform's
        // roots (or SSL_CERT_FILE) for this host.
        let roots = crate::net::x509::roots::system().map_err(|e| fail(url, "tls", e))?;
        let verifier = crate::net::x509::verify::WebVerifier { roots: &roots.certs, now: None };
        let config = crate::net::tls::Config { host, verifier: &verifier };
        Box::new(crate::net::tls::connect(coop, &config).map_err(|e| tls_fail(url, e))?)
    };
    Ok(Idle { key, stream, limits, probe, since: Instant::now(), keep: POOL_IDLE })
}

/// Sends the request on `idle` and reads the response head; the body is
/// read here, or with `stream` left in the returned `Streaming`. A connection
/// the server leaves open goes back to the pool when `pool` allows. The
/// error's flag says the connection failed before any response byte arrived.
fn run(mut idle: Idle, request: &[u8], method: &str, url: &str, key: &(bool, String, u16), pool: bool, stream: bool) -> Result<Reply, (NetFail, bool)> {
    if let Err(e) = idle.stream.write_all(request).and_then(|()| idle.stream.flush()) {
        return Err((fail(url, connect_kind(&e), e.to_string()), true));
    }
    let mut r = BufReader::new(idle.stream);
    let mut seen = false;
    let head = match read_head(&mut r, method, &mut seen) {
        Ok(h) => h,
        Err((kind, message)) => return Err((fail(url, kind, message), !seen)),
    };
    let keep = head
        .headers
        .iter()
        .find(|(k, _)| k == "keep-alive")
        .and_then(|(_, v)| v.split(',').find_map(|p| p.trim().strip_prefix("timeout=")?.parse::<u64>().ok()))
        .map_or(POOL_IDLE, |s| Duration::from_secs(s.min(30)));
    let mut st = Streaming {
        r: Some(r),
        frame: head.frame,
        reusable: head.reusable && pool,
        idle: Some((key.clone(), idle.limits, idle.probe, keep)),
    };
    let mut resp = Response { status: head.status, headers: head.headers, all: head.all, body: Vec::new() };
    if stream {
        // A body of no length is finished already: give the connection back now.
        if matches!(st.frame, Frame::None) {
            st.finish();
        }
        return Ok(Reply { resp, open: Some(st) });
    }
    resp.body = st.read_all().map_err(|(kind, message)| (fail(url, kind, message), false))?;
    Ok(Reply { resp, open: None })
}

/// std.net.4: a TLS failure is kind `tls` with its cause; a transport
/// failure during the handshake keeps its usual kind.
fn tls_fail(url: &str, e: crate::net::tls::TlsError) -> NetFail {
    use crate::net::tls::Kind;
    match e.kind {
        Kind::Io(k) => fail(url, connect_kind(&std::io::Error::from(k)), e.message),
        Kind::Protocol | Kind::Certificate => fail(url, "tls", e.message),
    }
}

fn connect_kind(e: &std::io::Error) -> &'static str {
    use std::io::ErrorKind as K;
    match e.kind() {
        K::ConnectionRefused => "refused",
        K::TimedOut | K::WouldBlock => "timed_out",
        // Only the TLS stream reports InvalidData: a protocol failure after
        // the handshake (std.net.4).
        K::InvalidData => "tls",
        _ => "other",
    }
}

type Bad = (&'static str, String);

fn protocol(message: impl Into<String>) -> Bad {
    ("protocol", message.into())
}

fn read_line(r: &mut impl BufRead) -> Result<String, Bad> {
    let mut line = Vec::new();
    r.read_until(b'\n', &mut line).map_err(|e| (connect_kind(&e), e.to_string()))?;
    let text = String::from_utf8_lossy(&line);
    Ok(text.trim_end_matches(['\r', '\n']).to_string())
}

/// Read a response to `method` (RFC 9112 message framing).
#[cfg(test)]
fn read_response(r: impl BufRead, method: &str) -> Result<Response, Bad> {
    let mut r = BufReader::new(r);
    let head = read_head(&mut r, method, &mut false)?;
    let mut st = Streaming { r: Some(BufReader::new(Box::new(std::io::empty()) as Stream)), frame: Frame::None, reusable: false, idle: None };
    // Feed the body from the same reader.
    let body = read_frame_all(&mut r, head.frame)?;
    st.finish();
    Ok(Response { status: head.status, headers: head.headers, all: head.all, body })
}

/// How a response body is delimited (RFC 9112 6.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Frame {
    /// No body (`HEAD`, `1xx`, `204`, `304`, or a length of 0 once read).
    None,
    /// This many bytes are left.
    Length(usize),
    /// Chunked; `left` bytes remain in the current chunk, `0` between chunks.
    Chunked { left: usize },
    /// Up to the end of the connection.
    ToEof,
}

struct Head {
    status: i64,
    headers: Vec<(String, String)>,
    all: Vec<(String, Vec<String>)>,
    frame: Frame,
    /// HTTP/1.1, no `Connection: close`, and a delimited body.
    reusable: bool,
}

/// Reads the status line and headers, skipping interim `1xx` responses
/// (except 101) (std.net.19). `seen` turns true when the first byte arrives.
fn read_head(r: &mut impl BufRead, method: &str, seen: &mut bool) -> Result<Head, Bad> {
    let mut status_line = read_line(r)?;
    *seen = !status_line.is_empty();
    loop {
        let interim = status_line.split(' ').nth(1).and_then(|s| s.parse::<i64>().ok()).is_some_and(|s| (100..200).contains(&s) && s != 101);
        if !interim {
            break;
        }
        while !read_line(r)?.is_empty() {}
        status_line = read_line(r)?;
    }
    let mut parts = status_line.splitn(3, ' ');
    let version = parts.next().unwrap_or("");
    if !version.starts_with("HTTP/1.") {
        return Err(protocol(format!("not an HTTP/1 response: `{status_line}`")));
    }
    let status: i64 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| protocol(format!("no status code in `{status_line}`")))?;
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut all: Vec<(String, Vec<String>)> = Vec::new();
    loop {
        let line = read_line(r)?;
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(protocol(format!("bad header line `{line}`")));
        };
        let (name, value) = (name.trim().to_ascii_lowercase(), value.trim().to_string());
        match all.iter_mut().find(|(k, _)| *k == name) {
            Some((_, vs)) => vs.push(value.clone()),
            None => all.push((name.clone(), vec![value.clone()])),
        }
        match headers.iter_mut().find(|(k, _)| *k == name) {
            Some((_, v)) => {
                v.push_str(", ");
                v.push_str(&value);
            }
            None => headers.push((name, value)),
        }
    }
    let header = |n: &str| headers.iter().find(|(k, _)| k == n).map(|(_, v)| v.as_str());
    let no_body = method == "HEAD" || status / 100 == 1 || status == 204 || status == 304;
    let frame = if no_body {
        Frame::None
    } else if header("transfer-encoding").is_some_and(|v| v.to_ascii_lowercase().contains("chunked")) {
        Frame::Chunked { left: 0 }
    } else if let Some(len) = header("content-length") {
        let len: usize = len.parse().map_err(|_| protocol(format!("bad Content-Length `{len}`")))?;
        if len == 0 { Frame::None } else { Frame::Length(len) }
    } else {
        Frame::ToEof
    };
    let closes = header("connection").is_some_and(|v| v.to_ascii_lowercase().contains("close"));
    let reusable = frame != Frame::ToEof && !closes && version == "HTTP/1.1";
    Ok(Head { status, headers, all, frame, reusable })
}

/// The parts of a pooled connection that travel with a body being read.
type IdleParts = ((bool, String, u16), Limits, TcpStream, Duration);

/// A response body being read in pieces (std.net.25). When it has been read
/// to its end on a connection that can carry another request, the connection
/// goes back to the pool; dropped earlier, the connection is closed.
pub struct Streaming {
    r: Option<BufReader<Stream>>,
    frame: Frame,
    reusable: bool,
    idle: Option<IdleParts>,
}

impl Streaming {
    /// Up to `max` bytes of the body; none at its end.
    pub fn read_some(&mut self, max: usize) -> Result<Vec<u8>, Bad> {
        let Some(r) = self.r.as_mut() else { return Ok(Vec::new()) };
        let io = |e: std::io::Error| (connect_kind(&e), e.to_string());
        let mut out = Vec::new();
        match self.frame {
            Frame::None => {}
            Frame::Length(left) => {
                out.resize(max.min(left), 0);
                let n = r.read(&mut out).map_err(io)?;
                if n == 0 {
                    return Err(("closed", "the connection ended before the whole body arrived".to_string()));
                }
                out.truncate(n);
                self.frame = if n == left { Frame::None } else { Frame::Length(left - n) };
            }
            Frame::Chunked { mut left } => {
                if left == 0 {
                    let size_line = read_line(r)?;
                    let size_text = size_line.split(';').next().unwrap_or("").trim();
                    left = usize::from_str_radix(size_text, 16).map_err(|_| protocol(format!("bad chunk size `{size_line}`")))?;
                    if left == 0 {
                        // Trailers, up to the blank line.
                        while !read_line(r)?.is_empty() {}
                        self.frame = Frame::None;
                        self.finish();
                        return Ok(out);
                    }
                }
                out.resize(max.min(left), 0);
                let n = r.read(&mut out).map_err(io)?;
                if n == 0 {
                    return Err(("closed", "the connection ended inside a chunk".to_string()));
                }
                out.truncate(n);
                left -= n;
                if left == 0 {
                    read_line(r)?;
                }
                self.frame = Frame::Chunked { left };
            }
            Frame::ToEof => {
                out.resize(max, 0);
                let n = r.read(&mut out).map_err(io)?;
                out.truncate(n);
                if n == 0 {
                    self.frame = Frame::None;
                }
            }
        }
        if self.frame == Frame::None && self.r.is_some() {
            self.finish();
        }
        Ok(out)
    }

    /// The rest of the body.
    pub fn read_all(&mut self) -> Result<Vec<u8>, Bad> {
        let mut body = Vec::new();
        while self.r.is_some() {
            let more = self.read_some(64 * 1024)?;
            body.extend_from_slice(&more);
        }
        Ok(body)
    }

    /// The body is over: the connection returns to the pool if it can.
    fn finish(&mut self) {
        let Some(r) = self.r.take() else { return };
        if let Some((key, limits, probe, keep)) = self.idle.take()
            && self.reusable
            && r.buffer().is_empty()
        {
            give_back(Idle { key, stream: r.into_inner(), limits, probe, since: Instant::now(), keep });
        }
    }
}

/// The body of a response whose head `read_head` has read, from `r` itself.
#[cfg(test)]
fn read_frame_all(r: &mut impl BufRead, frame: Frame) -> Result<Vec<u8>, Bad> {
    let mut body = Vec::new();
    let io = |e: std::io::Error| (connect_kind(&e), e.to_string());
    match frame {
        Frame::None => {}
        Frame::Length(n) => {
            body.resize(n, 0);
            r.read_exact(&mut body).map_err(io)?;
        }
        Frame::Chunked { .. } => loop {
            let size_line = read_line(r)?;
            let size = usize::from_str_radix(size_line.split(';').next().unwrap_or("").trim(), 16)
                .map_err(|_| protocol(format!("bad chunk size `{size_line}`")))?;
            if size == 0 {
                while !read_line(r)?.is_empty() {}
                break;
            }
            let start = body.len();
            body.resize(start + size, 0);
            r.read_exact(&mut body[start..]).map_err(io)?;
            read_line(r)?;
        },
        Frame::ToEof => {
            r.read_to_end(&mut body).map_err(io)?;
        }
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls() {
        let t = parse_url("http://example.com:8080/a/b?q=1#frag").ok().unwrap();
        assert_eq!((t.host.as_str(), t.port, t.path.as_str()), ("example.com", 8080, "/a/b?q=1"));
        let t = parse_url("http://localhost").ok().unwrap();
        assert_eq!((t.port, t.path.as_str()), (80, "/"));
        let t = parse_url("https://example.com/x").ok().unwrap();
        assert_eq!((t.port, t.tls, t.path.as_str()), (443, true, "/x"));
        assert_eq!(parse_url("HTTPS://example.com:8443").ok().unwrap().port, 8443);
        assert_eq!(parse_url("ftp://example.com/").err().unwrap().kind, "bad_url");
        assert_eq!(parse_url("example.com").err().unwrap().kind, "bad_url");
        assert_eq!(resolve("http://h:1/a/b", "/c"), "http://h:1/c");
        assert_eq!(resolve("http://h/a/b", "c"), "http://h/a/c");
    }

    #[test]
    fn chunked_and_headers() {
        let raw = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nX-A: 1\r\nx-a: 2\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n";
        let r = read_response(raw.as_bytes(), "GET").ok().unwrap();
        assert_eq!(r.status, 200);
        assert_eq!(r.body, b"Wikipedia");
        assert_eq!(r.headers[1], ("x-a".to_string(), "1, 2".to_string()));
    }
}
