//! `std.net` (spec 17-std, ADR 0283): a minimal HTTP/1.1 client on the
//! standard library's TCP, with `https` over the in-house TLS 1.3 client
//! (std.net.5, ADRs 0360, 0369). Request options and the cooperative
//! transport are std.net.6-8 (ADRs 0418-0420).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::netsock::Coop;

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
    let method = method.to_ascii_uppercase();
    if method.is_empty() || !method.bytes().all(is_token_byte) {
        return Err(fail(url, "bad_request", format!("`{method}` is not an HTTP method")));
    }
    for (name, value) in &opts.headers {
        check_header(url, name, value)?;
    }
    let mut url = with_query(url, &opts.query);
    let mut opts = opts.clone();
    let follows = method == "GET" || method == "HEAD";
    for _ in 0..=opts.max_redirects {
        let r = exchange_once(&method, &url, &opts)?;
        let location = r.headers.iter().find(|(k, _)| k == "location").map(|(_, v)| v.clone());
        match (r.status, location) {
            (301 | 302 | 303 | 307 | 308, Some(to)) if follows && opts.max_redirects > 0 => {
                let next = resolve(&url, &to);
                if origin(&next) != origin(&url) {
                    opts.headers.retain(|(k, _)| !k.eq_ignore_ascii_case("authorization") && !k.eq_ignore_ascii_case("cookie"));
                }
                url = next;
            }
            _ => return Ok(r),
        }
    }
    Err(fail(&url, "protocol", format!("more than {} redirects", opts.max_redirects)))
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

fn exchange_once(method: &str, url: &str, opts: &Opts) -> Result<Response, NetFail> {
    let t = parse_url(url)?;
    let io = |kind: &'static str, e: std::io::Error| fail(url, kind, e.to_string());
    let host = t.host.trim_start_matches('[').trim_end_matches(']');
    let addrs: Vec<_> = (host, t.port)
        .to_socket_addrs()
        .map_err(|e| fail(url, "not_found", format!("can't find host `{}`: {e}", t.host)))?
        .collect();
    let mut last = None;
    let mut stream = None;
    for a in &addrs {
        match TcpStream::connect_timeout(a, opts.timeout) {
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
    let default_port = if t.tls { 443 } else { 80 };
    let host_header = if t.port == default_port { t.host.clone() } else { format!("{}:{}", t.host, t.port) };
    let given = |name: &str| opts.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(name));
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
    head.push_str("Connection: close\r\n");
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
    if !opts.body.is_empty() || matches!(method, "POST" | "PUT" | "PATCH") {
        head.push_str(&format!("Content-Length: {}\r\n", opts.body.len()));
    }
    head.push_str("\r\n");
    let mut bytes = head.into_bytes();
    bytes.extend_from_slice(&opts.body);
    // The stream waits cooperatively when tasks are running, so a client and
    // a server in one program both make progress (std.net.8).
    let stream = Coop::for_client(stream, opts.timeout).map_err(|e| io("other", e))?;
    if !t.tls {
        return exchange(stream, &bytes, method, url);
    }
    // std.net.5: TLS 1.3, the certificate checked against the platform's
    // roots (or SSL_CERT_FILE) for this host.
    let roots = crate::net::x509::roots::system().map_err(|e| fail(url, "tls", e))?;
    let verifier = crate::net::x509::verify::WebVerifier { roots: &roots.certs, now: None };
    let config = crate::net::tls::Config { host, verifier: &verifier };
    let tls = crate::net::tls::connect(stream, &config).map_err(|e| tls_fail(url, e))?;
    exchange(tls, &bytes, method, url)
}

/// Sends the request and reads the response over a plain or TLS stream.
fn exchange<S: Read + Write>(mut stream: S, request: &[u8], method: &str, url: &str) -> Result<Response, NetFail> {
    stream.write_all(request).map_err(|e| fail(url, connect_kind(&e), e.to_string()))?;
    read_response(BufReader::new(stream), method).map_err(|(kind, message)| fail(url, kind, message))
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
fn read_response(mut r: impl BufRead, method: &str) -> Result<Response, Bad> {
    let status_line = read_line(&mut r)?;
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
        let line = read_line(&mut r)?;
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
    let mut body = Vec::new();
    let no_body = method == "HEAD" || status / 100 == 1 || status == 204 || status == 304;
    if no_body {
        // Framed by the status alone.
    } else if header("transfer-encoding").is_some_and(|v| v.to_ascii_lowercase().contains("chunked")) {
        loop {
            let size_line = read_line(&mut r)?;
            let size_text = size_line.split(';').next().unwrap_or("").trim();
            let size = usize::from_str_radix(size_text, 16)
                .map_err(|_| protocol(format!("bad chunk size `{size_line}`")))?;
            if size == 0 {
                // Trailers, up to the blank line.
                while !read_line(&mut r)?.is_empty() {}
                break;
            }
            let start = body.len();
            body.resize(start + size, 0);
            r.read_exact(&mut body[start..]).map_err(|e| (connect_kind(&e), e.to_string()))?;
            read_line(&mut r)?;
        }
    } else if let Some(len) = header("content-length") {
        let len: usize = len.parse().map_err(|_| protocol(format!("bad Content-Length `{len}`")))?;
        body.resize(len, 0);
        r.read_exact(&mut body).map_err(|e| (connect_kind(&e), e.to_string()))?;
    } else {
        r.read_to_end(&mut body).map_err(|e| (connect_kind(&e), e.to_string()))?;
    }
    Ok(Response { status, headers, all, body })
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
