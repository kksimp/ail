//! The client handshake (RFC 8446 §2, §4) and the stream it leaves behind.

use std::io::{self, Read, Write};

use super::codec::{Reader, handshake, put_vec};
use super::record::{ALERT, APPLICATION_DATA, CHANGE_CIPHER_SPEC, HANDSHAKE, RecordLayer};
use super::schedule::{Secrets, Suite, TrafficKeys, Transcript};
use super::{SIGNATURE_SCHEMES, TlsError, Verifier, alert_name};
use crate::net::crypto::{ct, rand, x25519};

// Handshake message types (§4).
const CLIENT_HELLO: u8 = 1;
const SERVER_HELLO: u8 = 2;
const NEW_SESSION_TICKET: u8 = 4;
const ENCRYPTED_EXTENSIONS: u8 = 8;
const CERTIFICATE: u8 = 11;
const CERTIFICATE_REQUEST: u8 = 13;
const CERTIFICATE_VERIFY: u8 = 15;
const FINISHED: u8 = 20;
const KEY_UPDATE: u8 = 24;

// Extensions (§4.2).
const SERVER_NAME: u16 = 0;
const SUPPORTED_GROUPS: u16 = 10;
const SIGNATURE_ALGORITHMS: u16 = 13;
const COOKIE: u16 = 44;
const SUPPORTED_VERSIONS: u16 = 43;
const KEY_SHARE: u16 = 51;

const X25519: u16 = 0x001d;
const TLS13: u16 = 0x0304;

/// ServerHello.random of a HelloRetryRequest: SHA-256("HelloRetryRequest").
const RETRY_RANDOM: [u8; 32] = [
    0xcf, 0x21, 0xad, 0x74, 0xe5, 0x9a, 0x61, 0x11, 0xbe, 0x1d, 0x8c, 0x02, 0x1e, 0x65, 0xb8, 0x91, 0xc2, 0xa2, 0x11,
    0x16, 0x7a, 0xbb, 0x8c, 0x5e, 0x07, 0x9e, 0x09, 0xe2, 0xc8, 0xa8, 0x33, 0x9c,
];

/// What a connection needs besides the stream.
pub struct Config<'a> {
    /// The server's name, sent as SNI and checked against its certificate.
    pub host: &'a str,
    pub verifier: &'a dyn Verifier,
}

/// The randomness a handshake consumes: the hello random, the legacy
/// session id, and the X25519 private key. Tests fix them.
struct Ephemeral {
    random: [u8; 32],
    session_id: [u8; 32],
    secret: [u8; 32],
}

impl Ephemeral {
    fn fresh() -> Result<Ephemeral, TlsError> {
        let mut e = Ephemeral { random: [0; 32], session_id: [0; 32], secret: [0; 32] };
        for buf in [&mut e.random, &mut e.session_id, &mut e.secret] {
            rand::fill(buf).map_err(|err| TlsError::protocol(format!("no secure random numbers: {err}")))?;
        }
        Ok(e)
    }
}

/// An established TLS connection: `Read` gives the server's application
/// data and `Write` sends ours.
pub struct TlsStream<S> {
    records: RecordLayer<S>,
    suite: Suite,
    incoming: Vec<u8>,
    pos: usize,
    handshake_buf: Vec<u8>,
    closed: bool,
}

/// Performs the handshake over `stream` (§2): ClientHello, ServerHello
/// (with at most one HelloRetryRequest), the server's encrypted flight with
/// its certificate checked by `config.verifier`, and our Finished.
pub fn connect<S: Read + Write>(stream: S, config: &Config) -> Result<TlsStream<S>, TlsError> {
    handshake_with(stream, config, Ephemeral::fresh()?, None)
}

/// Builds a ClientHello (§4.1.2). `cookie` echoes a HelloRetryRequest's.
fn client_hello(host: &str, e: &Ephemeral, cookie: Option<&[u8]>) -> Vec<u8> {
    let mut body = Vec::with_capacity(512);
    body.extend_from_slice(&0x0303u16.to_be_bytes()); // legacy_version
    body.extend_from_slice(&e.random);
    put_vec(&mut body, 1, &e.session_id); // compatibility mode (§D.4)
    let suites: Vec<u8> = Suite::OFFERED.iter().flat_map(|s| s.code().to_be_bytes()).collect();
    put_vec(&mut body, 2, &suites);
    put_vec(&mut body, 1, &[0]); // legacy_compression_methods: null
    let mut ext = Vec::new();
    let mut add = |ty: u16, data: &[u8]| {
        ext.extend_from_slice(&ty.to_be_bytes());
        put_vec(&mut ext, 2, data);
    };
    // SNI is a DNS name, never an IP literal (RFC 6066 §3).
    if host.parse::<std::net::IpAddr>().is_err() {
        let mut name = vec![0]; // host_name
        put_vec(&mut name, 2, host.as_bytes());
        let mut list = Vec::new();
        put_vec(&mut list, 2, &name);
        add(SERVER_NAME, &list);
    }
    let mut versions = Vec::new();
    put_vec(&mut versions, 1, &TLS13.to_be_bytes());
    add(SUPPORTED_VERSIONS, &versions);
    let mut groups = Vec::new();
    put_vec(&mut groups, 2, &X25519.to_be_bytes());
    add(SUPPORTED_GROUPS, &groups);
    let schemes: Vec<u8> = SIGNATURE_SCHEMES.iter().flat_map(|s| s.to_be_bytes()).collect();
    let mut sig = Vec::new();
    put_vec(&mut sig, 2, &schemes);
    add(SIGNATURE_ALGORITHMS, &sig);
    let mut share = X25519.to_be_bytes().to_vec();
    put_vec(&mut share, 2, &x25519::public_key(&e.secret));
    let mut shares = Vec::new();
    put_vec(&mut shares, 2, &share);
    add(KEY_SHARE, &shares);
    if let Some(c) = cookie {
        let mut v = Vec::new();
        put_vec(&mut v, 2, c);
        add(COOKIE, &v);
    }
    put_vec(&mut body, 2, &ext);
    handshake(CLIENT_HELLO, &body)
}

/// A ServerHello or HelloRetryRequest, as far as this client needs it.
struct ServerHello {
    retry: bool,
    suite: Suite,
    /// The server's X25519 share (ServerHello), or the group it asks for
    /// (HelloRetryRequest).
    share: Option<Vec<u8>>,
    asked_group: Option<u16>,
    cookie: Option<Vec<u8>>,
}

fn parse_server_hello(body: &[u8], session_id: &[u8]) -> Result<ServerHello, TlsError> {
    let mut r = Reader::new(body, "ServerHello");
    let legacy_version = r.u16()?;
    let random = r.take(32)?;
    let retry = random == RETRY_RANDOM;
    let echoed = r.vec8()?;
    let code = r.u16()?;
    let compression = r.u8()?;
    let mut ext = Reader::new(if r.is_empty() { &[] } else { r.vec16()? }, "ServerHello extensions");
    r.finish()?;
    let mut version = None;
    let mut share = None;
    let mut asked_group = None;
    let mut cookie = None;
    let mut seen = Vec::new();
    while !ext.is_empty() {
        let ty = ext.u16()?;
        let data = ext.vec16()?;
        if seen.contains(&ty) {
            return Err(TlsError::protocol(format!("the server repeated extension {ty} (illegal_parameter)")));
        }
        seen.push(ty);
        let mut d = Reader::new(data, "ServerHello extension");
        match ty {
            SUPPORTED_VERSIONS => version = Some(d.u16()?),
            KEY_SHARE if retry => asked_group = Some(d.u16()?),
            KEY_SHARE => {
                let group = d.u16()?;
                if group != X25519 {
                    return Err(TlsError::protocol(format!(
                        "the server chose key exchange group {group:#06x}, which wasn't offered (illegal_parameter)"
                    )));
                }
                share = Some(d.vec16()?.to_vec());
            }
            COOKIE if retry => cookie = Some(d.vec16()?.to_vec()),
            _ => {
                return Err(TlsError::protocol(format!(
                    "the server sent extension {ty} in its ServerHello, which wasn't offered (unsupported_extension)"
                )));
            }
        }
        d.finish()?;
    }
    if version != Some(TLS13) {
        // A TLS 1.2 server answers without supported_versions (§4.1.3).
        return Err(TlsError::protocol(match version {
            None if legacy_version <= 0x0303 => {
                "the server only speaks TLS 1.2 or older; AIL supports TLS 1.3 only (ADR 0360)".to_string()
            }
            v => format!("the server chose TLS version {:#06x}, not 1.3 (protocol_version)", v.unwrap_or(legacy_version)),
        }));
    }
    if echoed != session_id {
        return Err(TlsError::protocol("the server didn't echo the session id (illegal_parameter)"));
    }
    if compression != 0 {
        return Err(TlsError::protocol("the server chose compression (illegal_parameter)"));
    }
    let suite = Suite::from_code(code).ok_or_else(|| {
        TlsError::protocol(format!("the server chose cipher suite {code:#06x}, which wasn't offered (illegal_parameter)"))
    })?;
    Ok(ServerHello { retry, suite, share, asked_group, cookie })
}

/// Handshake messages reassembled from records (§5.1: they may span or
/// share records).
struct Messages {
    buf: Vec<u8>,
}

impl Messages {
    /// The next complete handshake message (header included), reading
    /// records as needed. Unprotected change_cipher_spec records are
    /// skipped while `allow_ccs`.
    fn next<S: Read + Write>(&mut self, rl: &mut RecordLayer<S>, allow_ccs: bool) -> Result<Vec<u8>, TlsError> {
        loop {
            if self.buf.len() >= 4 {
                let len = (self.buf[1] as usize) << 16 | (self.buf[2] as usize) << 8 | self.buf[3] as usize;
                if len > 1 << 17 {
                    return Err(TlsError::protocol(format!("the server sent a {len}-byte handshake message")));
                }
                if self.buf.len() >= 4 + len {
                    return Ok(self.buf.drain(..4 + len).collect());
                }
            }
            let (ty, data) = rl.read()?;
            match ty {
                HANDSHAKE if !data.is_empty() => self.buf.extend_from_slice(&data),
                CHANGE_CIPHER_SPEC if allow_ccs && data == [1] => {}
                ALERT => return Err(alert_error(&data)),
                _ => {
                    return Err(TlsError::protocol(format!(
                        "the server sent a record of type {ty} in the middle of the handshake (unexpected_message)"
                    )));
                }
            }
        }
    }

    /// §5.1: handshake messages must not span a key change.
    fn expect_boundary(&self) -> Result<(), TlsError> {
        if self.buf.is_empty() {
            Ok(())
        } else {
            Err(TlsError::protocol("the server's handshake message spanned a key change (unexpected_message)"))
        }
    }
}

fn alert_error(data: &[u8]) -> TlsError {
    match data {
        [_, 0] => TlsError::protocol("the server closed the connection during the handshake"),
        [_, code] => {
            let name = alert_name(*code);
            if *code == 70 {
                TlsError::protocol(
                    "the server doesn't support TLS 1.3 (alert protocol_version); AIL supports TLS 1.3 only (ADR 0360)",
                )
            } else if *code == 40 {
                TlsError::protocol(
                    "the server refused the handshake (alert handshake_failure); it may need a key exchange group or cipher suite AIL's TLS lacks",
                )
            } else if matches!(code, 42..=46 | 48) {
                TlsError::certificate(format!("the server rejected the connection: {name}"))
            } else {
                TlsError::protocol(format!("the server sent the alert {name} ({code})"))
            }
        }
        _ => TlsError::protocol("the server sent a malformed alert"),
    }
}

fn expect(msg: &[u8], ty: u8, name: &str) -> Result<(), TlsError> {
    if msg[0] == ty {
        Ok(())
    } else {
        Err(TlsError::protocol(format!(
            "the server sent handshake message {} where {name} belongs (unexpected_message)",
            msg[0]
        )))
    }
}

/// The handshake with given randomness. `hello_override` replaces the
/// first ClientHello's bytes (tests replaying a recorded trace whose key
/// share is `e.secret`'s).
fn handshake_with<S: Read + Write>(
    stream: S,
    config: &Config,
    e: Ephemeral,
    hello_override: Option<Vec<u8>>,
) -> Result<TlsStream<S>, TlsError> {
    let mut rl = RecordLayer::new(stream);
    let mut transcript = Transcript::default();
    let mut messages = Messages { buf: Vec::new() };
    let session_id = match &hello_override {
        Some(h) => {
            // legacy_session_id follows type(1) len(3) version(2) random(32).
            let n = h[38] as usize;
            h[39..39 + n].to_vec()
        }
        None => e.session_id.to_vec(),
    };
    let ch = hello_override.unwrap_or_else(|| client_hello(config.host, &e, None));
    rl.write(HANDSHAKE, &ch)?;
    transcript.add(&ch);

    let mut sh_msg = messages.next(&mut rl, false)?;
    expect(&sh_msg, SERVER_HELLO, "ServerHello")?;
    let mut sh = parse_server_hello(&sh_msg[4..], &session_id)?;
    if sh.retry {
        // §4.1.4: the only reason we'd retry is a cookie; a different group
        // can't be met because X25519 is all this client has.
        match sh.asked_group {
            Some(X25519) => {
                return Err(TlsError::protocol(
                    "the server asked again for X25519, which was already offered (illegal_parameter)",
                ));
            }
            Some(g) => {
                return Err(TlsError::protocol(format!(
                    "the server requires key exchange group {g:#06x}; AIL's TLS supports only X25519"
                )));
            }
            None if sh.cookie.is_none() => {
                return Err(TlsError::protocol("the server's HelloRetryRequest asked for nothing (illegal_parameter)"));
            }
            None => {}
        }
        let first_suite = sh.suite;
        transcript.restart_after_retry(sh.suite);
        transcript.add(&sh_msg);
        let ch2 = client_hello(config.host, &e, sh.cookie.as_deref());
        rl.write(HANDSHAKE, &ch2)?;
        transcript.add(&ch2);
        sh_msg = messages.next(&mut rl, true)?;
        expect(&sh_msg, SERVER_HELLO, "ServerHello")?;
        sh = parse_server_hello(&sh_msg[4..], &session_id)?;
        if sh.retry {
            return Err(TlsError::protocol("the server sent a second HelloRetryRequest (unexpected_message)"));
        }
        if sh.suite != first_suite {
            return Err(TlsError::protocol("the server changed cipher suite after HelloRetryRequest (illegal_parameter)"));
        }
    }
    transcript.add(&sh_msg);
    let suite = sh.suite;
    let peer: [u8; 32] = sh
        .share
        .ok_or_else(|| TlsError::protocol("the server sent no key share (missing_extension)"))?
        .try_into()
        .map_err(|_| TlsError::protocol("the server's X25519 key share has the wrong length (illegal_parameter)"))?;
    let shared = x25519::x25519(&e.secret, &peer);
    // RFC 8446 §7.4.2 / RFC 7748 §6.1: an all-zero result means a
    // low-order point.
    if ct::eq(&shared, &[0; 32]) {
        return Err(TlsError::protocol("the server's X25519 key share is invalid (illegal_parameter)"));
    }
    let secrets = Secrets::handshake(suite, &shared, &transcript.hash(suite));
    messages.expect_boundary()?;
    rl.read_keys = Some(TrafficKeys::new(suite, &secrets.server_handshake));

    // The server's encrypted flight (§4.3–4.4).
    let ee = messages.next(&mut rl, true)?;
    expect(&ee, ENCRYPTED_EXTENSIONS, "EncryptedExtensions")?;
    let mut r = Reader::new(&ee[4..], "EncryptedExtensions");
    let _extensions = r.vec16()?; // SNI acknowledgement, max_fragment_length, ...: nothing to act on
    r.finish()?;
    transcript.add(&ee);

    let mut msg = messages.next(&mut rl, true)?;
    let mut cert_requested: Option<Vec<u8>> = None;
    if msg[0] == CERTIFICATE_REQUEST {
        // We have no client certificate; we'll answer with an empty one (§4.4.2).
        let mut r = Reader::new(&msg[4..], "CertificateRequest");
        cert_requested = Some(r.vec8()?.to_vec());
        transcript.add(&msg);
        msg = messages.next(&mut rl, true)?;
    }
    expect(&msg, CERTIFICATE, "Certificate")?;
    let chain = parse_certificate(&msg[4..])?;
    transcript.add(&msg);

    let cv = messages.next(&mut rl, true)?;
    expect(&cv, CERTIFICATE_VERIFY, "CertificateVerify")?;
    let mut r = Reader::new(&cv[4..], "CertificateVerify");
    let scheme = r.u16()?;
    let signature = r.vec16()?;
    r.finish()?;
    if !SIGNATURE_SCHEMES[..5].contains(&scheme) {
        return Err(TlsError::protocol(format!(
            "the server signed with scheme {scheme:#06x}, which wasn't offered (illegal_parameter)"
        )));
    }
    let mut signed = vec![0x20u8; 64];
    signed.extend_from_slice(b"TLS 1.3, server CertificateVerify\0");
    signed.extend_from_slice(&transcript.hash(suite));
    config.verifier.verify(config.host, &chain, scheme, &signed, signature)?;
    transcript.add(&cv);

    let fin = messages.next(&mut rl, true)?;
    expect(&fin, FINISHED, "Finished")?;
    let want = suite.finished(&secrets.server_handshake, &transcript.hash(suite));
    if !ct::eq(&fin[4..], &want) {
        return Err(TlsError::protocol("the server's Finished didn't verify (decrypt_error)"));
    }
    transcript.add(&fin);
    messages.expect_boundary()?;
    let (client_app, server_app) = secrets.application(suite, &transcript.hash(suite));

    // Our flight: change_cipher_spec for middleboxes (§D.4), then
    // (an empty Certificate if asked) and Finished under handshake keys.
    rl.write(CHANGE_CIPHER_SPEC, &[1])?;
    rl.write_keys = Some(TrafficKeys::new(suite, &secrets.client_handshake));
    if let Some(context) = cert_requested {
        let mut body = Vec::new();
        put_vec(&mut body, 1, &context);
        put_vec(&mut body, 3, &[]);
        let m = handshake(CERTIFICATE, &body);
        rl.write(HANDSHAKE, &m)?;
        transcript.add(&m);
    }
    let ours = handshake(FINISHED, &suite.finished(&secrets.client_handshake, &transcript.hash(suite)));
    rl.write(HANDSHAKE, &ours)?;
    rl.read_keys = Some(TrafficKeys::new(suite, &server_app));
    rl.write_keys = Some(TrafficKeys::new(suite, &client_app));
    Ok(TlsStream { records: rl, suite, incoming: Vec::new(), pos: 0, handshake_buf: Vec::new(), closed: false })
}

/// The DER certificates of a Certificate message, leaf first (§4.4.2).
fn parse_certificate(body: &[u8]) -> Result<Vec<Vec<u8>>, TlsError> {
    let mut r = Reader::new(body, "Certificate");
    let context = r.vec8()?;
    if !context.is_empty() {
        return Err(TlsError::protocol("the server's Certificate has a request context (illegal_parameter)"));
    }
    let mut list = Reader::new(r.vec24()?, "Certificate");
    r.finish()?;
    let mut chain = Vec::new();
    while !list.is_empty() {
        let cert = list.vec24()?;
        let _extensions = list.vec16()?; // OCSP staples and SCTs aren't checked
        if cert.is_empty() {
            return Err(TlsError::protocol("the server sent an empty certificate (decode_error)"));
        }
        chain.push(cert.to_vec());
    }
    if chain.is_empty() {
        return Err(TlsError::certificate("the server sent no certificate"));
    }
    Ok(chain)
}

impl<S: Read + Write> TlsStream<S> {
    /// The negotiated cipher suite's IANA name.
    pub fn suite(&self) -> &'static str {
        self.suite.name()
    }

    /// Reads records until application data arrives or the server closes.
    /// Handles post-handshake messages (§4.6): tickets are ignored, and a
    /// KeyUpdate rolls our read keys (and our write keys if asked).
    fn fill(&mut self) -> Result<(), TlsError> {
        while self.pos == self.incoming.len() && !self.closed {
            let (ty, data) = match self.records.read() {
                Ok(r) => r,
                // Servers often drop the connection without close_notify
                // after `Connection: close`; HTTP framing notices truncation.
                Err(e) if e.kind == super::Kind::Io(io::ErrorKind::UnexpectedEof) => {
                    self.closed = true;
                    return Ok(());
                }
                Err(e) => return Err(e),
            };
            match ty {
                APPLICATION_DATA => {
                    self.incoming = data;
                    self.pos = 0;
                }
                ALERT => match data.as_slice() {
                    [_, 0] => self.closed = true,
                    [_, code] => {
                        return Err(TlsError::protocol(format!(
                            "the server sent the alert {} ({code})",
                            alert_name(*code)
                        )));
                    }
                    _ => return Err(TlsError::protocol("the server sent a malformed alert")),
                },
                HANDSHAKE => {
                    self.handshake_buf.extend_from_slice(&data);
                    self.post_handshake()?;
                }
                _ => {
                    return Err(TlsError::protocol(format!(
                        "the server sent a record of type {ty} after the handshake (unexpected_message)"
                    )));
                }
            }
        }
        Ok(())
    }

    fn post_handshake(&mut self) -> Result<(), TlsError> {
        while self.handshake_buf.len() >= 4 {
            let b = &self.handshake_buf;
            let len = (b[1] as usize) << 16 | (b[2] as usize) << 8 | b[3] as usize;
            if len > 1 << 17 {
                return Err(TlsError::protocol("the server sent an oversized post-handshake message"));
            }
            if b.len() < 4 + len {
                return Ok(());
            }
            let msg: Vec<u8> = self.handshake_buf.drain(..4 + len).collect();
            match msg[0] {
                NEW_SESSION_TICKET => {} // no resumption (ADR 0367)
                KEY_UPDATE => {
                    if !self.handshake_buf.is_empty() {
                        return Err(TlsError::protocol("a KeyUpdate spanned a key change (unexpected_message)"));
                    }
                    let requested = match &msg[4..] {
                        [0] => false,
                        [1] => true,
                        _ => return Err(TlsError::protocol("the server sent a malformed KeyUpdate (decode_error)")),
                    };
                    let next = self.records.read_keys.as_ref().unwrap().next();
                    self.records.read_keys = Some(next);
                    if requested {
                        self.records.write(HANDSHAKE, &handshake(KEY_UPDATE, &[0]))?;
                        let next = self.records.write_keys.as_ref().unwrap().next();
                        self.records.write_keys = Some(next);
                    }
                }
                other => {
                    return Err(TlsError::protocol(format!(
                        "the server sent handshake message {other} after the handshake (unexpected_message)"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Sends close_notify (§6.1). Errors are ignored: the peer may already
    /// have gone.
    pub fn close(&mut self) {
        let _ = self.records.write(ALERT, &[1, 0]);
    }
}

fn to_io(e: TlsError) -> io::Error {
    match e.kind {
        super::Kind::Io(kind) => io::Error::new(kind, e.message),
        _ => io::Error::new(io::ErrorKind::InvalidData, e),
    }
}

impl std::error::Error for TlsError {}

impl<S: Read + Write> Read for TlsStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.fill().map_err(to_io)?;
        let n = buf.len().min(self.incoming.len() - self.pos);
        buf[..n].copy_from_slice(&self.incoming[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

impl<S: Read + Write> Write for TlsStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.records.write(APPLICATION_DATA, buf).map_err(to_io)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.records.stream.flush()
    }
}

#[cfg(test)]
pub(crate) mod tests;
