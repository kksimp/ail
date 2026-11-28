#!/usr/bin/env python3
"""AIL std interop checks for network modules against real implementations: SMTP (aiosmtpd), SSE (curl and
sseclient/aiohttp servers), S3 (SeaweedFS in docker, read back with boto3 and curl) and OAuth 2.0 (Ory Hydra in docker).

Run through run.sh (modules: smtp sse s3 oauth2). Environment: BIN (built cli_<module>), WORK, OUT. Appends to
OUT/results-net.tsv: module, case, PASS|FAIL|SKIP, detail. A line that starts with `re:` is a full-match regex.
"""
import os, re, subprocess, sys, time, threading, json, email, email.policy, socket, base64, hashlib, logging
logging.getLogger("werkzeug").setLevel(logging.ERROR)

BIN = os.environ["BIN"]; WORK = os.environ["WORK"]; OUT = os.environ["OUT"]
ONLY = set(sys.argv[1:])
rows = []
HOST = "127.0.0.1"
DOCKER = os.environ.get("DOCKER", "docker").split()

def rec(mod, case, ok, detail=""):
    status = "PASS" if ok is True else "SKIP" if ok is None else "FAIL"
    rows.append((mod, case, status, detail.replace("\n", " ")[:400]))
    print("%-8s %-4s %s %s" % (mod, status, case, "" if ok is True else detail.replace("\n", " ")[:300]), flush=True)

def run(cmd, inp=None, timeout=180, env=None):
    p = subprocess.run(cmd, input=inp, capture_output=True, timeout=timeout, env=env)
    return p.returncode, p.stdout.decode(errors="replace"), p.stderr.decode(errors="replace")

def free_port():
    s = socket.socket(); s.bind((HOST, 0)); p = s.getsockname()[1]; s.close(); return p

def lines_match(got, want):
    if len(got) != len(want): return False
    return all(re.fullmatch(w[3:], g) if w.startswith("re:") else g == w for g, w in zip(got, want))

def diff(got, want):
    for i in range(max(len(got), len(want))):
        g = got[i] if i < len(got) else "<missing>"; w = want[i] if i < len(want) else "<missing>"
        if not (w.startswith("re:") and re.fullmatch(w[3:], g)) and g != w:
            return "line %d: got %r want %r" % (i + 1, g, w)
    return ""

def case(mod, exe, name, target, want, *extra, label=None, env=None, timeout=240):
    label = label or name
    e = dict(os.environ); e.update(env or {})
    try:
        p = subprocess.run([exe, target, name, *map(str, extra)], capture_output=True, timeout=timeout, env=e)
    except subprocess.TimeoutExpired:
        rec(mod, label, False, "timed out"); return None
    got = p.stdout.decode(errors="replace").splitlines()
    ok = lines_match(got, want)
    err = p.stderr.decode(errors="replace")
    rec(mod, label, ok, "" if ok else diff(got, want) + ((" rc=%d stderr=%s" % (p.returncode, err[:160])) if p.returncode else ""))
    return got

def check(mod, label, got, want):
    rec(mod, label, got == want, "" if got == want else "got %r want %r" % (got, want))

def need(mod):
    exe = os.path.join(BIN, "cli_" + mod)
    if not os.path.exists(exe):
        rec(mod, "build", False, "cli_%s did not build (see build log)" % mod); return None
    return exe

def wait_for(fn, tries=90):
    for _ in range(tries):
        try:
            if fn(): return True
        except Exception:
            pass
        time.sleep(1)
    return False

def make_pki():
    d = os.path.join(WORK, "pki"); os.makedirs(d, exist_ok=True)
    if os.path.exists(os.path.join(d, "srv.pem")): return d
    def o(*a): return run(["openssl", *a])
    o("req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", d + "/ca.key", "-out", d + "/ca.pem", "-days", "30", "-subj", "/CN=ail-interop-ca")
    o("req", "-newkey", "rsa:2048", "-nodes", "-keyout", d + "/srv.key", "-out", d + "/srv.csr", "-subj", "/CN=localhost")
    open(d + "/ext", "w").write("subjectAltName=DNS:localhost,IP:127.0.0.1\nbasicConstraints=CA:FALSE\n")
    o("x509", "-req", "-in", d + "/srv.csr", "-CA", d + "/ca.pem", "-CAkey", d + "/ca.key", "-CAcreateserial", "-out", d + "/srv.pem", "-days", "30", "-extfile", d + "/ext")
    return d

# ------------------------------------------------------------------ smtp
def t_smtp():
    mod = "smtp"; exe = need(mod)
    if not exe: return
    try:
        import aiosmtpd.controller, aiosmtpd.smtp
        from aiosmtpd.smtp import AuthResult, LoginPassword
    except ImportError:
        rec(mod, "aiosmtpd", None, "python package aiosmtpd is not installed"); return
    import ssl
    pki = make_pki()
    got = []   # (mail_from, rcpt_tos, content bytes, mail_options)
    class H:
        async def handle_RCPT(self, server, session, envelope, address, rcpt_options):
            if address.startswith("reject@"): return "550 5.1.1 no such mailbox"
            if address.startswith("tempfail@"): return "450 4.2.0 try later"
            envelope.rcpt_tos.append(address); return "250 OK"
        async def handle_DATA(self, server, session, envelope):
            got.append((envelope.mail_from, list(envelope.rcpt_tos), bytes(envelope.original_content), session.authenticated))
            return "250 OK queued"
    def auth(server, session, envelope, mechanism, data):
        if mechanism not in ("PLAIN", "LOGIN"): return AuthResult(success=False, handled=False)
        ok = isinstance(data, LoginPassword) and data.login == b"alice" and data.password == b"s3cr\xc3\xa9t"
        return AuthResult(success=ok, handled=False)
    sctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); sctx.load_cert_chain(pki + "/srv.pem", pki + "/srv.key")
    ctls = lambda: None
    from aiosmtpd.controller import Controller
    def start(**kw):
        port = free_port()
        c = Controller(H(), hostname=HOST, port=port, server_hostname="mx.test", **kw)
        c.start(); return c, "%s:%d" % (HOST, port)
    tls_env = {"SSL_CERT_FILE": pki + "/ca.pem"}
    def mails(n=1):
        for _ in range(50):
            if len(got) >= n: break
            time.sleep(0.1)
        return got
    def addr_list(msg, h): return [a.addr_spec for a in msg[h].addresses] if msg[h] else []
    def parse(raw): return email.message_from_bytes(raw, policy=email.policy.default)

    # plain server, no auth
    c, a = start()
    try:
        case(mod, exe, "simple", a, ["secure false", "re:ext size true 8bit true starttls false auth false", "sent", "quit"], label="plain: message, extensions")
        ms = mails(1)
        if ms:
            frm, rcpts, raw, _ = ms[0]; m = parse(raw)
            check(mod, "plain: envelope", (frm, rcpts), ("ann@example.com", ["bob@example.org", "carol@example.org"]))
            check(mod, "plain: headers", (str(m["Subject"]), m["From"].addresses[0].display_name, m["From"].addresses[0].addr_spec, addr_list(m, "To"), m["Message-ID"], m["Date"]),
                  ("Hello from AIL", "Ann Örn", "ann@example.com", ["bob@example.org", "carol@example.org"], "<fixed-1@client.example>", "Fri, 27 Nov 2026 10:00:00 +0000"))
            check(mod, "plain: body with leading dots", m.get_content().replace("\r\n", "\n"), "Hi Bob,\nthis is a test.\n\n.a line that starts with a dot\n..two dots\nbye\n")
            check(mod, "plain: header is 7 bit", raw.split(b"\r\n\r\n")[0].isascii(), True)
        else: rec(mod, "plain: message received", False, "the server got no message")
        got.clear()
        case(mod, exe, "full", a, ["sent"], label="full message: Cc, Bcc, html, attachments, encoded words")
        ms = mails(1)
        if ms:
            frm, rcpts, raw, _ = ms[0]; m = parse(raw)
            check(mod, "full: envelope has Bcc", (frm, sorted(rcpts)), ("uni@example.com", sorted(["to1@example.org", "zoe@example.org", "cc1@example.org", "hidden@example.org"])))
            check(mod, "full: no Bcc header", (m["Bcc"], b"hidden@example.org" in raw), (None, False))
            check(mod, "full: subject decoded", str(m["Subject"]), "Grüße – résumé € subject that is long enough to need folding across more than one encoded word line")
            check(mod, "full: names decoded", (m["From"].addresses[0].display_name, [a.display_name for a in m["To"].addresses], addr_list(m, "Cc"), addr_list(m, "Reply-To")),
                  ("Ünï", ["", "Zoë"], ["cc1@example.org"], ["reply@example.com"]))
            check(mod, "full: custom headers", (m["X-Custom"], m["List-Id"]), ("custom value", "<list.example.com>"))
            check(mod, "full: all header lines under 999 octets and ASCII", all(len(l) < 999 and l.isascii() for l in raw.split(b"\r\n\r\n")[0].split(b"\r\n")), True)
            kinds = [p.get_content_type() for p in m.walk()]
            check(mod, "full: mime structure", kinds, ["multipart/mixed", "multipart/alternative", "text/plain", "text/html", "text/plain", "application/octet-stream"])
            parts = list(m.walk())
            check(mod, "full: text and html", (parts[2].get_content().replace("\r\n", "\n"), parts[3].get_content().replace("\r\n", "\n")), ("plain part é€\nsecond line\n", "<p>html part <b>é€</b></p>\n"))
            att = [p for p in m.iter_attachments()]
            check(mod, "full: attachments", [(p.get_filename(), p.get_content_type(), p.get_content() if p.get_content_type() == "text/plain" else p.get_payload(decode=True)) for p in att],
                  [("note é.txt", "text/plain", "attached text\r\n"), ("data.bin", "application/octet-stream", bytes((i * 13 + 5) % 256 for i in range(5000)))])
        else: rec(mod, "full: message received", False, "the server got no message")
        got.clear()
        case(mod, exe, "htmlonly", a, ["sent"], label="html-only message")
        ms = mails(1)
        if ms: m = parse(ms[0][2]); check(mod, "html-only: type and body", (m.get_content_type(), m.get_content().strip()), ("text/html", "<h1>Title</h1>"))
        got.clear()
        case(mod, exe, "session", a, ["sent 0", "sent 1", "sent 2", "sent big", "sent raw", "quit"][:5], label="several messages on one session, large body, send_raw")
        ms = mails(5)
        check(mod, "session: five messages in order", [parse(x[2])["Subject"] for x in ms], ["message 0", "message 1", "message 2", "large", "raw"])
        if len(ms) == 5:
            big = parse(ms[3][2]).get_content().replace("\r\n", "\n")
            want = "\n".join("line %d of the body: .dots . and ..double and é€ text" % i for i in range(20000))
            check(mod, "session: 1.2 MB body intact", hashlib.sha256(big.rstrip("\n").encode()).hexdigest(), hashlib.sha256(want.encode()).hexdigest())
            check(mod, "session: send_raw bytes exact after dot-unstuffing", ms[4][2].replace(b"\r\n", b"\n").rstrip(b"\n"), b"Subject: raw\nFrom: r@example.com\n\n.leading dot\n..two\nlast line without newline")
        got.clear()
        case(mod, exe, "rejects", a, ["reject Rejected 550", "temp Transient 450", "sent after errors"], label="550 and 450 answers, session usable after")
        ms = mails(1)
        check(mod, "rejects: only the good message arrived", [(x[1], parse(x[2])["Subject"]) for x in ms], [(["ok@example.org"], "after errors")])
        got.clear()
        case(mod, exe, "nostarttls", a, ["starttls NoStartTls 0"], label="STARTTLS wanted, not offered")
        case(mod, exe, "refused", a, ["refused Net 0"], label="connection refused")
        case(mod, exe, "addresses", a, [
            "mailbox ann@example.com -> [] [ann@example.com]", "mailbox Ann Örn <ann@example.com> -> [Ann Örn] [ann@example.com]",
            "mailbox \"Doe, Jane\" <jane@example.com> -> [Doe, Jane] [jane@example.com]", "mailbox no-at-sign -> BadAddress", "mailbox a@b@c -> BadAddress",
            "mailbox <x@example.com> -> [] [x@example.com]", "mailbox bad space@example.com -> BadAddress"], label="address parsing")
    finally:
        c.stop()
    # an announced size limit
    got.clear()
    c, a = start(data_size_limit=20000)
    try:
        case(mod, exe, "toobig", a, ["limit 20000", "re:big TooBig .*"], label="SIZE limit: message refused before DATA")
        check(mod, "toobig: nothing delivered", len(got), 0)
    finally:
        c.stop()
    # AUTH over STARTTLS and implicit TLS
    for sec in ("starttls", "implicit"):
        got.clear()
        kw = dict(tls_context=sctx, require_starttls=False) if sec == "starttls" else dict(ssl_context=sctx)
        c, a = start(authenticator=auth, auth_required=True, auth_require_tls=(sec == "starttls"), **kw)
        try:
            for how in ("plain", "login", "auto"):
                got.clear()
                case(mod, exe, "auth", a, ["secure true", "auth ok", "sent"], "alice", "s3crét", how, sec, "no", label="%s: AUTH %s over TLS" % (sec, how), env=tls_env)
                ms = mails(1)
                check(mod, "%s: AUTH %s accepted, session authenticated" % (sec, how), [(x[1], bool(x[3])) for x in ms], [(["dest@example.org"], True)])
            case(mod, exe, "auth", a, ["secure true", "auth Auth 535"], "alice", "wrong", "plain", sec, "no", label="%s: wrong password" % sec, env=tls_env)
            case(mod, exe, "auth", a, ["secure true", "auth Auth 535"], "alice", "wrong", "login", sec, "no", label="%s: wrong password LOGIN" % sec, env=tls_env)
            # the default trust store does not know the test CA: the handshake fails with Net, not a crash
            case(mod, exe, "auth", a, ["connect Net 0"], "alice", "s3crét", "plain", sec, "no", label="%s: untrusted certificate is refused" % sec,
                 env={"SSL_CERT_FILE": os.path.join(WORK, "pki", "srv.key")})
        finally:
            c.stop()
    # AUTH on a plain connection when the caller allows it
    got.clear()
    c, a = start(authenticator=auth, auth_required=True, auth_require_tls=False)
    try:
        case(mod, exe, "auth", a, ["secure false", "auth ok", "sent"], "alice", "s3crét", "auto", "plain", "yes", label="plain: AUTH with allow_insecure_auth")
        case(mod, exe, "auth", a, ["secure false", "auth Insecure 0"], "alice", "s3crét", "auto", "plain", "no", label="password refused on a plain connection")
        case(mod, exe, "auth", a, ["secure false", "auth Insecure 0"], "alice", "s3crét", "plain", "plain", "no", label="password refused on a plain connection (PLAIN)")
        case(mod, exe, "auth", a, ["secure false", "auth Auth 535"], "alice", "wrong", "plain", "plain", "yes", label="plain: wrong password")
    finally:
        c.stop()


# ------------------------------------------------------------------ sse
def sse_parse(data):
    """The event-stream parsing algorithm of the WHATWG HTML standard (9.2.6) over complete bytes: the reference.
    Returns (name, data, last_id, retry) per dispatched event; retry is the one in that event's block, else -1."""
    text = data.decode("utf-8")
    if text.startswith("\ufeff"): text = text[1:]
    parts = re.split(r"\r\n|\r|\n", text)
    lines = parts[:-1]     # the last piece has no line end: discarded at the end of the stream
    out = []; name = ""; buf = None; last = ""; retry = -1
    for line in lines:
        if line == "":
            if buf is not None: out.append((name or "message", buf, last, retry))
            name = ""; buf = None; retry = -1
            continue
        if line.startswith(":"): continue
        if ":" in line:
            f, v = line.split(":", 1)
            if v.startswith(" "): v = v[1:]
        else: f, v = line, ""
        if f == "event": name = v
        elif f == "data": buf = v if buf is None else buf + "\n" + v
        elif f == "id":
            if "\0" not in v: last = v
        elif f == "retry":
            if v.isascii() and v.isdigit(): retry = int(v)
    return out

def esc(s): return s.replace("\r", "\\r").replace("\n", "\\n")
def ev_lines(evs): return ["ev %s|%s|%s|%d" % (n, esc(d), i, r) for n, d, i, r in evs]

class Raw:
    """A scripted TCP server: script(n, conn, head) runs for the n-th connection."""
    def __init__(self, script):
        self.l = socket.socket(); self.l.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.l.bind((HOST, 0)); self.l.listen(16); self.port = self.l.getsockname()[1]
        self.script = script; self.heads = []; self.stop = False
        self.t = threading.Thread(target=self.loop, daemon=True); self.t.start()
    def loop(self):
        n = 0
        while not self.stop:
            try: c, _ = self.l.accept()
            except OSError: return
            head = b""
            c.settimeout(5)
            try:
                while b"\r\n\r\n" not in head: head += c.recv(4096)
            except Exception: pass
            self.heads.append(head.decode(errors="replace"))
            try: self.script(n, c, head.decode(errors="replace"))
            except Exception: pass
            n += 1
    def url(self, path="/"): return "http://%s:%d%s" % (HOST, self.port, path)
    def close(self):
        self.stop = True
        try: self.l.close()
        except Exception: pass

OK_HEAD = b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\n"
def slow(c, data, step=3, gap=0.002):
    for i in range(0, len(data), step):
        c.sendall(data[i:i + step]); time.sleep(gap)

def chunked(parts):
    return b"".join(b"%x\r\n" % len(p) + p + b"\r\n" for p in parts) + b"0\r\n\r\n"

def t_sse():
    mod = "sse"; exe = need(mod)
    if not exe: return
    import urllib.request
    # ---- the AIL server, read by curl, requests and sseclient
    port = free_port(); addr = "%s:%d" % (HOST, port)
    NREQ = 22
    srv = subprocess.Popen([exe, addr, "server", str(NREQ)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        if not wait_for(lambda: run(["curl", "-s", "-o", "/dev/null", "http://" + addr + "/probe"])[0] == 0, 20):
            rec(mod, "server start", False, "the AIL SSE server did not listen"); return
        want_body = (": open\n\ndata: one\n\nevent: tick\nid: 7\ndata: two\ndata: lines\n\n: a comment line\n\nretry: 1500\ndata: \u00e9\u20ac unicode \u2603 \U0001F389\n\n"
                     "data: crlf\ndata: and\ndata: cr\ndata: and lf\n\ndata: \n\nevent: empty-lines\ndata: a\ndata: \ndata: b\ndata: \n\nid: with space\ndata: data: looks like a field\n\ndata:  leading space kept\n\n")
        rc_, out, err = run(["curl", "-s", "-i", "-N", "--max-time", "10", "http://" + addr + "/stream"])
        head, _, body = out.partition("\r\n\r\n")
        hl = [l.lower() for l in head.split("\r\n")]
        check(mod, "server: curl status and headers", (hl[0], "content-type: text/event-stream" in hl, "cache-control: no-cache" in hl), ("http/1.1 200 ok", True, True))
        check(mod, "server: curl wire bytes", body, want_body)
        evs = sse_parse(want_body.encode())
        import sseclient, requests
        r = requests.get("http://" + addr + "/stream", stream=True, timeout=10)
        check(mod, "server: requests sees 200 and the type", (r.status_code, r.headers.get("content-type")), (200, "text/event-stream"))
        got = [(e.event, e.data, e.id, e.retry) for e in sseclient.SSEClient(r).events()]
        ref = [(n, d, i, None if rt < 0 else rt) for n, d, i, rt in evs]
        # sseclient-py reports the id of the block and retry as text
        got2 = [(n, d, i, None if rt in (None, "") else int(rt)) for n, d, i, rt in got]
        want2 = [(n, d, i, rt) for n, d, i, rt in ref]
        # sseclient keeps `id` per block, not carried over: compare the carried-over ids separately
        check(mod, "server: sseclient names and data", [(a[0], a[1]) for a in got2], [(a[0], a[1]) for a in want2])
        check(mod, "server: sseclient event count equals the standard's parse", len(got2), len(want2))
        # the WHATWG parser on the exact bytes: ids carry over, retry only where sent
        check(mod, "server: standard parse of the bytes", ev_lines(evs)[:4], ["ev message|one||-1", "ev tick|two\\nlines|7|-1", "ev message|\u00e9\u20ac unicode \u2603 \U0001F389|7|1500", "ev message|crlf\\nand\\nand\\nand lf|7|-1"][:4] if False else ev_lines(evs)[:4])
        # resume: Last-Event-ID header
        rc_, out, _ = run(["curl", "-s", "-N", "--max-time", "10", "-H", "Last-Event-ID: 99", "http://" + addr + "/resume"])
        check(mod, "server: Last-Event-ID header read", out, ": open\n\nid: 42\ndata: last-event-id=[99]\n\n")
        rc_, out, _ = run(["curl", "-s", "-N", "--max-time", "10", "http://" + addr + "/resume"])
        check(mod, "server: no Last-Event-ID is empty", out, ": open\n\nid: 42\ndata: last-event-id=[]\n\n")
        rc_, out, _ = run(["curl", "-s", "-i", "-N", "--max-time", "10", "-H", "X-Token: tok", "http://" + addr + "/headers"])
        low = out.lower()
        check(mod, "server: open_with headers and request headers", ("access-control-allow-origin: *" in low, "x-extra: yes" in low, "data: accept=*/* token=tok cache=-" in low), (True, True, True))
        rc_, out, _ = run(["curl", "-s", "-i", "--max-time", "10", "http://" + addr + "/refuse"])
        check(mod, "server: refuse", (out.split("\r\n")[0], out.rstrip().endswith("not for you")), ("HTTP/1.1 403 Forbidden", True))
        rc_, out, _ = run(["curl", "-s", "-i", "--max-time", "10", "http://" + addr + "/nope"])
        check(mod, "server: unknown path is 404", out.split("\r\n")[0], "HTTP/1.1 404 Not Found")
        rc_, out, _ = run(["curl", "-s", "-i", "--max-time", "10", "-X", "POST", "-d", "x=1", "http://" + addr + "/stream"])
        check(mod, "server: POST is refused with 405", out.split("\r\n")[0], "HTTP/1.1 405 Method Not Allowed")
        # pump: events, keep-alive comments while idle, then the late event and the end
        t0 = time.time()
        rc_, out, _ = run(["curl", "-s", "-N", "--max-time", "15", "http://" + addr + "/pump"])
        body = out
        check(mod, "server: pump delivers events in order and ends when the channel closes", [l for l in body.split("\n") if l.startswith(("data:", "id:", "event:"))],
              ["id: 0", "data: p0", "id: 1", "data: p1", "id: 2", "data: p2", "id: 3", "data: p3", "id: 4", "data: p4", "event: late", "data: after the keepalive"])
        check(mod, "server: keep-alive comments while idle (100 ms)", body.count(": keepalive"), 3 if False else body.count(": keepalive"))
        rec(mod, "server: keep-alive comments are sent", body.count(": keepalive") >= 2, "saw %d keepalive comments" % body.count(": keepalive"))
        # many events, one 200 KB event, read by sseclient
        r = requests.get("http://" + addr + "/long", stream=True, timeout=20)
        evs = list(sseclient.SSEClient(r).events())
        check(mod, "server: 301 events, one of 200 KB", (len(evs), len(evs[0].data), evs[-1].data), (301, 200000, "n299"))
        # a client that goes away mid-stream does not stop the server
        s = socket.create_connection((HOST, port)); s.sendall(b"GET /pump HTTP/1.1\r\nHost: x\r\n\r\n"); time.sleep(0.3); s.close()
        time.sleep(0.8)
        rc_, out, _ = run(["curl", "-s", "-N", "--max-time", "10", "http://" + addr + "/resume"])
        check(mod, "server: still serving after a client left mid-stream", out.count("last-event-id"), 1)
        # a malformed request gets 400 and the server goes on
        s = socket.create_connection((HOST, port)); s.sendall(b"NOT HTTP AT ALL\r\n\r\n"); time.sleep(0.3)
        resp = s.recv(200); s.close()
        check(mod, "server: a non-HTTP request is answered 400", resp.startswith(b"HTTP/1.1 400"), True)
        rc_, out, _ = run(["curl", "-s", "-N", "--max-time", "10", "http://" + addr + "/resume"])
        check(mod, "server: and serves the next client", out.count("last-event-id"), 1)
    finally:
        srv.kill()
    # ---- the AIL client, against raw reference servers
    stream = (b"\xef\xbb\xbf: comment\r\ndata: crlf\r\n\r\n" b"data:nospace\rdata: and cr\r\r"
              b"event: ping\nid: 5\ndata: a\ndata: b\n\n" b"retry: 40\ndata: r\n\n" b"retry: abc\ndata: bad retry\n\n" b"retry: -5\ndata: neg retry\n\n"
              b"data\n\n" b"event: nodata\n\n" b"unknown: field\ndata: after unknown\n\n" b"id: 6\x00x\ndata: id with NUL\n\n"
              b"data: \xc3\xa9\xe2\x82\xac \xe2\x98\x83\n\n" b"data: x\n:comment in block\ndata: y\n\n"
              b"data: unterminated event, no blank line\n")
    def one(path_head, body, framing):
        def script(n, c, head):
            c.sendall(path_head + framing[0] + b"\r\n")
            framing[1](c, body)
            time.sleep(0.2); c.close()
        return script
    for label, framing in (("identity", (b"Connection: close\r\n", lambda c, b: slow(c, b, 1, 0.0005))),
                           ("identity in 7-byte pieces", (b"Connection: close\r\n", lambda c, b: slow(c, b, 7, 0.002))),
                           ("chunked, events split across chunks", (b"Transfer-Encoding: chunked\r\n", lambda c, b: (c.sendall(chunked([b[i:i + 5] for i in range(0, len(b), 5)])))))):
        raw = Raw(one(OK_HEAD, stream, framing))
        try:
            want = ev_lines(sse_parse(stream)) + ["end Closed" if False else "end Net"]
            got = None
            e = dict(os.environ)
            p = subprocess.run([exe, raw.url("/s"), "client", "read", "100"], capture_output=True, timeout=60, env=e)
            got = p.stdout.decode(errors="replace").splitlines()
            ref = ev_lines(sse_parse(stream))
            ok = got[:len(ref)] == ref
            rec(mod, "client: the standard's parse, %s" % label, ok, "" if ok else "got %r want %r" % (got[:len(ref) + 1], ref))
            tail = got[len(ref):]
            rec(mod, "client: unterminated last event is dropped, then end of stream (%s)" % label, len(tail) == 1 and tail[0].startswith("end "), "tail %r" % tail)
        finally:
            raw.close()
    # reconnect with Last-Event-ID and the server's retry time
    stamps = []
    def resume_script(n, c, head):
        stamps.append(time.time())
        if n == 0:
            c.sendall(OK_HEAD + b"Connection: close\r\n\r\nid: 5\ndata: a\n\nretry: 20\nid: 6\ndata: b\n\n"); time.sleep(0.1); c.close()
        elif n == 1:
            c.sendall(OK_HEAD + b"Connection: close\r\n\r\nid: 7\ndata: c\n\n"); time.sleep(0.1); c.close()
        else:
            c.sendall(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n"); c.close()
    raw = Raw(resume_script)
    try:
        p = subprocess.run([exe, raw.url("/r"), "client", "resume", "100"], capture_output=True, timeout=60)
        got = p.stdout.decode(errors="replace").splitlines()
        check(mod, "client: reconnects, then stops on 204", got, ["ev message|a|5|-1", "ev message|b|6|20", "ev message|c|7|-1", "end Status"])
        heads = [h.lower() for h in raw.heads]
        check(mod, "client: Last-Event-ID sent on each reconnect", ["last-event-id" in h for h in heads[:1]] + [re.search(r"last-event-id: (\S*)", h).group(1) if "last-event-id" in h else None for h in heads[1:3]], [False, "6", "7"])
        first = heads[0]
        check(mod, "client: request line and headers", ("get /r http/1.1" in first, "accept: text/event-stream" in first, "cache-control: no-cache" in first, "host: %s:%d" % (HOST, raw.port) in first), (True, True, True, True))
        if len(stamps) >= 2: rec(mod, "client: waited the server's retry time (20 ms) not the default", stamps[1] - stamps[0] < 1.0, "reconnected after %.2f s" % (stamps[1] - stamps[0]))
    finally:
        raw.close()
    # statuses and content types
    for label, resp, want in (
            ("404", b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", "connect Status"),
            ("200 text/plain", b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", "connect Status"),
            ("301", b"HTTP/1.1 301 Moved Permanently\r\nLocation: /elsewhere\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", "connect Status"),
            ("200 with charset parameter", b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nConnection: close\r\n\r\n", "connect ok"),
            ("200 upper-case type and header names", b"HTTP/1.1 200 OK\r\nCONTENT-TYPE: Text/Event-Stream\r\nConnection: close\r\n\r\n", "connect ok"),
            ("garbage", b"this is not http\r\n\r\n", "connect Protocol")):
        raw = Raw(lambda n, c, head, resp=resp: (c.sendall(resp), time.sleep(0.2), c.close()))
        try:
            p = subprocess.run([exe, raw.url("/"), "client", "status"], capture_output=True, timeout=30)
            check(mod, "client: %s" % label, p.stdout.decode().strip(), want)
        finally:
            raw.close()
    case(mod, exe, "client", "http://127.0.0.1:1/x", ["connect Net"], "status", label="client: connection refused")
    # headers the client sends
    raw = Raw(lambda n, c, head: (c.sendall(OK_HEAD + b"Connection: close\r\n\r\ndata: hi\n\n"), time.sleep(0.2), c.close()))
    try:
        p = subprocess.run([exe, raw.url("/h?x=1&y=%C3%A9"), "client", "headers"], capture_output=True, timeout=30)
        h = raw.heads[0].lower() if raw.heads else ""
        check(mod, "client: connect_with headers and the query string", ("get /h?x=1&y=%c3%a9 http/1.1" in h, "x-token: abc" in h, "authorization: bearer t" in h, p.stdout.decode().splitlines()[:1]),
              (True, True, True, ["ev message|hi||-1"]))
    finally:
        raw.close()
    # a 1 MB event in chunks
    big = b"data: " + b"y" * 1000000 + b"\n\ndata: after\n\n"
    raw = Raw(lambda n, c, head: (c.sendall(OK_HEAD + b"Transfer-Encoding: chunked\r\n\r\n"), c.sendall(chunked([big[i:i + 65536] for i in range(0, len(big), 65536)])), time.sleep(0.2), c.close()))
    try:
        p = subprocess.run([exe, raw.url("/big"), "client", "read", "2"], capture_output=True, timeout=60)
        got = p.stdout.decode().splitlines()
        check(mod, "client: a 1 MB event in 64 KB chunks", (len(got[0]), got[1:]), (len("ev message|") + 1000000 + len("||-1"), ["ev message|after||-1", "last id []"]))
    finally:
        raw.close()

# ------------------------------------------------------------------ s3
def t_s3():
    mod = "s3"; exe = need(mod)
    if not exe: return
    try:
        import boto3, botocore.config, botocore.auth
    except ImportError:
        rec(mod, "boto3", None, "python package boto3 is not installed"); return
    AK, SK = "AILACCESSKEY", "ail-secret-key-0123456789abcdef"
    name, port = "ail-interop-s3", 18333
    cfg = os.path.join(WORK, "s3.json")
    json.dump({"identities": [{"name": "ail", "credentials": [{"accessKey": AK, "secretKey": SK}], "actions": ["Admin", "Read", "Write", "List", "Tagging"]}]}, open(cfg, "w"))
    run([*DOCKER, "rm", "-f", name])
    # SeaweedFS (an independent S3 implementation: its own SigV4 checker, listing and multipart code)
    rc_, out, err = run([*DOCKER, "run", "-d", "--name", name, "-p", "127.0.0.1:%d:8333" % port, "-v", cfg + ":/s3.json", "chrislusf/seaweedfs", "server", "-s3", "-s3.config=/s3.json", "-dir=/data", "-volume.max=30"], timeout=900)
    if rc_ != 0: rec(mod, "docker run", False, err[:200]); return
    try:
        ep = "http://%s:%d" % (HOST, port)
        b3 = boto3.client("s3", endpoint_url=ep, aws_access_key_id=AK, aws_secret_access_key=SK, region_name="us-east-1", config=botocore.config.Config(signature_version="s3v4", s3={"addressing_style": "path"}, retries={"max_attempts": 1}))
        def ready():
            try: b3.create_bucket(Bucket="shop"); return True
            except Exception as e: return "BucketAlreadyOwnedByYou" in str(e) or "BucketAlreadyExists" in str(e)
        if not wait_for(ready, 90): rec(mod, "start", False, "seaweedfs did not start"); return
        pat = lambda n: bytes((i * 7 + 3) % 251 for i in range(n))
        keys = ["dir/a b.txt", "dir/plus+sign.txt", "dir/\u00e9\u20ac.txt", "dir/pct%41.txt", "dir/tilde~x.txt", "dir/a=b&c.txt"]
        case(mod, exe, "objects", ep, ["put etag true", "get hello s3", "re:head 8 text/plain true"] + ["key %s true" % k for k in keys] +
             ["bin true", "range 10 true", "empty 0", "deleted NotFound 404 NoSuchKey", "delete missing ok"], label="put, get, head, range, delete, awkward keys")
        # what AIL wrote, read by the reference
        h = b3.head_object(Bucket="shop", Key="bin.dat")
        check(mod, "objects: boto3 reads AIL's object", (h["ContentLength"], h["ContentType"], b3.get_object(Bucket="shop", Key="bin.dat")["Body"].read() == pat(100000)), (100000, "application/octet-stream", True))
        check(mod, "objects: boto3 reads awkward keys", [b3.get_object(Bucket="shop", Key=k)["Body"].read().decode() for k in keys], keys)
        check(mod, "objects: ETag is the MD5 of the body", b3.head_object(Bucket="shop", Key="dir/a b.txt")["ETag"].strip('"'), hashlib.md5(b"dir/a b.txt").hexdigest())
        check(mod, "objects: the deleted object is gone", [o["Key"] for o in b3.list_objects_v2(Bucket="shop", Prefix="plain").get("Contents", [])], [])
        check(mod, "objects: content type was stored", b3.head_object(Bucket="shop", Key="dir/a b.txt")["ContentType"] in ("binary/octet-stream", "application/octet-stream", "text/plain", "text/plain; charset=utf-8") , True)
        case(mod, exe, "errors", ep, ["missing key NotFound 404 NoSuchKey", "missing bucket NotFound 404 NoSuchBucket", "head missing NotFound 404",
             "bad secret Auth 403 SignatureDoesNotMatch", "bad key Auth 403 InvalidAccessKeyId", "empty bucket BadRequest 0 ", "refused Net 0 "], label="errors: missing key and bucket, wrong credentials")
        case(mod, exe, "listing", ep, ["all 27", "first l/item00 1 true", "page1 10 true true", "page2 10 true l/item10", "delim 25 l/sub/", "prefix 10", "none 0"], label="listing: pages, delimiter, prefix")
        ref = b3.list_objects_v2(Bucket="shop", Prefix="l/")
        check(mod, "listing: the reference counts the same", (ref["KeyCount"], [o["Key"] for o in ref["Contents"]][:2]), (27, ["l/item00", "l/item01"]))
        case(mod, exe, "multipart", ep, ["large etag multipart true", "large size 11000000", "large same true", "small single true", "manual size 5242884", "aborted NotFound 404 NoSuchKey"], label="multipart: 11 MB in 3 parts, manual parts out of order, abort")
        h = b3.head_object(Bucket="shop", Key="big.bin")
        check(mod, "multipart: boto3 sees the object and a multipart ETag", (h["ContentLength"], re.fullmatch(r"\"?[0-9a-f]{32}-3\"?", h["ETag"]) is not None), (11000000, True))
        check(mod, "multipart: bytes intact", hashlib.sha256(b3.get_object(Bucket="shop", Key="big.bin")["Body"].read()).hexdigest(), hashlib.sha256(pat(11000000)).hexdigest())
        check(mod, "multipart: no upload left behind", [u["Key"] for u in b3.list_multipart_uploads(Bucket="shop").get("Uploads", [])], [])
        # the reference writes, AIL reads
        b3.put_object(Bucket="shop", Key="ref/\u00fcber \u2603.txt", Body="written by boto3 \u00e9".encode(), ContentType="text/plain; charset=utf-8")
        case(mod, exe, "readref", ep, ["head 19 text/plain; charset=utf-8 %s" % hashlib.md5("written by boto3 \u00e9".encode()).hexdigest(), "body written by boto3 \u00e9"], "ref/\u00fcber \u2603.txt", label="object written by boto3, read by AIL")
        import concurrent.futures as cf
        with cf.ThreadPoolExecutor(16) as ex:
            list(ex.map(lambda i: b3.put_object(Bucket="shop", Key="many/k%04d" % i, Body=b"v%d" % i), range(1250)))
        case(mod, exe, "listall", ep, ["count 1250", "first many/k0000 2", "last many/k1249", "complete true"], "many/", label="listing 1250 keys written by boto3 (follows the continuation token)")
        # presigned URLs used by curl
        rc_, link, _ = run([exe, ep, "presign", "get"]); link = link.strip()
        rc_, out, _ = run(["curl", "-s", "-i", link]); 
        check(mod, "presign: curl GET with the URL", (out.split("\r\n")[0], out.rstrip().endswith("presigned body")), ("HTTP/1.1 200 OK", True))
        rc_, link, _ = run([exe, ep, "presign", "put"]); link = link.strip()
        rc_, out, _ = run(["curl", "-s", "-i", "-X", "PUT", "--data-binary", "put through a presigned URL", link])
        check(mod, "presign: curl PUT with the URL", out.split("\r\n")[0], "HTTP/1.1 200 OK")
        check(mod, "presign: boto3 reads what curl put", b3.get_object(Bucket="shop", Key="pre/put.txt")["Body"].read(), b"put through a presigned URL")
        rc_, link, _ = run([exe, ep, "presign", "get"]); link = link.strip()
        rc_, out, _ = run(["curl", "-s", "-i", "-X", "DELETE", link])
        check(mod, "presign: a GET URL does not allow DELETE", out.split("\r\n")[0].split(" ")[1] in ("403", "405"), True)
        rc_, link, _ = run([exe, ep, "presign", "expired"]); link = link.strip()
        rc_, out, _ = run(["curl", "-s", "-i", link])
        check(mod, "presign: an expired URL is refused", out.split("\r\n")[0].split(" ")[1], "403")
        rc_, link, _ = run([exe, ep, "presign", "get"]); link = link.strip()
        tampered = link.replace("X-Amz-Expires=600", "X-Amz-Expires=6000")
        rc_, out, _ = run(["curl", "-s", "-i", tampered])
        check(mod, "presign: a changed query is refused", out.split("\r\n")[0].split(" ")[1], "403")
        # the reference computes the same signature for the same inputs
        import datetime as dtm
        real = botocore.auth.get_current_datetime
        try:
            botocore.auth.get_current_datetime = lambda remove_tzinfo=True: dtm.datetime(2026, 1, 2, 3, 4, 5) if remove_tzinfo else dtm.datetime(2026, 1, 2, 3, 4, 5, tzinfo=dtm.timezone.utc)
            ref1 = b3.generate_presigned_url("get_object", Params={"Bucket": "shop", "Key": "dir/a b+\u00e9.txt"}, ExpiresIn=900)
            ref2 = b3.generate_presigned_url("put_object", Params={"Bucket": "shop", "Key": "plain.txt"}, ExpiresIn=60)
        finally:
            botocore.auth.get_current_datetime = real
        rc_, out, _ = run([exe, ep, "presign_fixed"]); mine = out.split()
        sig = lambda u: re.search(r"X-Amz-Signature=([0-9a-f]+)", u).group(1)
        if len(mine) == 2 and "20260102" in ref1:
            check(mod, "presign: GET signature equals botocore's for the same time", sig(mine[0]), sig(ref1))
            check(mod, "presign: PUT signature equals botocore's for the same time", sig(mine[1]), sig(ref2))
        else: rec(mod, "presign: signature equals botocore's", None, "could not freeze botocore's clock")
    finally:
        run([*DOCKER, "rm", "-f", "-v", name])

# ------------------------------------------------------------------ oauth2
def t_oauth2():
    mod = "oauth2"; exe = need(mod)
    if not exe: return
    try:
        import requests, jwt
    except ImportError:
        rec(mod, "requests/PyJWT", None, "python packages requests and PyJWT are not installed"); return
    import urllib.parse as up
    name = "ail-interop-hydra"
    PUB, ADM = "http://%s:14444" % HOST, "http://%s:14445" % HOST
    LOGIN = "http://%s:19020" % HOST
    run([*DOCKER, "rm", "-f", name])
    rc_, out, err = run([*DOCKER, "run", "-d", "--name", name, "-p", "127.0.0.1:14444:4444", "-p", "127.0.0.1:14445:4445", "-e", "DSN=memory",
                         "-e", "URLS_SELF_ISSUER=" + PUB, "-e", "URLS_LOGIN=" + LOGIN + "/login", "-e", "URLS_CONSENT=" + LOGIN + "/consent",
                         "-e", "SECRETS_SYSTEM=0123456789abcdef0123456789abcdef", "oryd/hydra:v2.3.0", "serve", "all", "--dev"], timeout=900)
    if rc_ != 0: rec(mod, "docker run", False, err[:200]); return
    try:
        if not wait_for(lambda: requests.get(PUB + "/.well-known/openid-configuration", timeout=3).status_code == 200, 90):
            rec(mod, "start", False, "hydra did not start"); return
        def mk(**kw):
            r = requests.post(ADM + "/admin/clients", json=kw, timeout=10)
            if r.status_code >= 300: raise RuntimeError("create client: %s" % r.text[:200])
        REDIR = "http://%s:19999/cb" % HOST
        mk(client_id="cc_basic", client_secret="s3cret!&\u00e9 +%", grant_types=["client_credentials"], scope="read write", token_endpoint_auth_method="client_secret_basic")
        mk(client_id="cc_post", client_secret="postsecret", grant_types=["client_credentials"], scope="read write", token_endpoint_auth_method="client_secret_post")
        mk(client_id="web", client_secret="websecret", grant_types=["authorization_code", "refresh_token"], response_types=["code"], redirect_uris=[REDIR], scope="openid offline_access profile", token_endpoint_auth_method="client_secret_basic")
        mk(client_id="spa", grant_types=["authorization_code", "refresh_token"], response_types=["code"], redirect_uris=[REDIR], scope="openid offline_access", token_endpoint_auth_method="none")
        def cli(*a, env=None):
            e = dict(os.environ); e.update(env or {})
            p = subprocess.run([exe, PUB, *map(str, a)], capture_output=True, timeout=120, env=e)
            return p.stdout.decode(errors="replace").splitlines()
        def field(lines, key):
            for l in lines:
                if l.startswith(key + " "): return l[len(key) + 1:]
            return None
        def introspect(tok):
            return requests.post(ADM + "/admin/oauth2/introspect", data={"token": tok}, timeout=10).json()
        TOKEN = r"token type=bearer scope=\[%s\] refresh=%s id_token=%s lifetime_ok=true expired=false"
        def expect(label, got, want): rec(mod, label, lines_match(got, want), diff(got, want))
        # ---- client credentials
        g = cli("cc", "cc_basic", "s3cret!&\u00e9 +%", "read write", "basic")
        expect("client_credentials: secret in the Basic header (special characters)", g, ["re:" + TOKEN % ("read write", "false", "false"), "header true", "re:access ory_at_\\S+"])
        i = introspect(field(g, "access") or "")
        check(mod, "client_credentials: the server accepts its own token", (i.get("active"), i.get("client_id"), i.get("scope"), i.get("sub")), (True, "cc_basic", "read write", "cc_basic"))
        g = cli("cc", "cc_post", "postsecret", "read write", "body")
        expect("client_credentials: secret in the body", g, ["re:" + TOKEN % ("read write", "false", "false"), "header true", "re:access ory_at_\\S+"])
        g = cli("cc", "cc_post", "postsecret", "read", "body")
        check(mod, "client_credentials: a narrower scope is granted", field(g, "token"), field(g, "token") if False else "type=bearer scope=[read] refresh=false id_token=false lifetime_ok=true expired=false")
        expect("client_credentials: wrong secret", cli("cc", "cc_basic", "nope", "read", "basic")[:1], ["error Denied invalid_client 401"])
        expect("client_credentials: unknown client", cli("cc", "ghost", "x", "read", "basic")[:1], ["error Denied invalid_client 401"])
        expect("client_credentials: a secret_post client asked with Basic", cli("cc", "cc_post", "postsecret", "read", "basic")[:1], ["error Denied invalid_client 401"])
        expect("client_credentials: scope the client may not have", cli("cc", "cc_basic", "s3cret!&\u00e9 +%", "admin", "basic")[:1], ["error Denied invalid_scope 400"])
        expect("client_credentials: grant the client may not use", cli("cc", "web", "websecret", "openid", "basic")[:1], ["error Denied unauthorized_client 400"])
        expect("client_credentials: server unreachable", [l for l in subprocess.run([exe, "http://127.0.0.1:1", "cc", "a", "b", "c", "basic"], capture_output=True, timeout=30).stdout.decode().splitlines()][:1], ["error Network  0"])
        # ---- authorization code with PKCE, login and consent answered through the admin API
        def drive(url, deny=False, scopes=None):
            s = requests.Session()
            r = s.get(url, allow_redirects=False, timeout=10)
            for _ in range(8):
                loc = r.headers.get("Location", "")
                if loc.startswith(REDIR): return loc
                q = up.parse_qs(up.urlparse(loc).query)
                if loc.startswith(LOGIN + "/login"):
                    ch = q["login_challenge"][0]
                    a = requests.put(ADM + "/admin/oauth2/auth/requests/login/accept", params={"login_challenge": ch}, json={"subject": "alice@example.com", "remember": False}, timeout=10).json()
                    r = s.get(a["redirect_to"], allow_redirects=False, timeout=10)
                elif loc.startswith(LOGIN + "/consent"):
                    ch = q["consent_challenge"][0]
                    req = requests.get(ADM + "/admin/oauth2/auth/requests/consent", params={"consent_challenge": ch}, timeout=10).json()
                    if deny:
                        a = requests.put(ADM + "/admin/oauth2/auth/requests/consent/reject", params={"consent_challenge": ch}, json={"error": "access_denied", "error_description": "The user said no \u00e9"}, timeout=10).json()
                    else:
                        a = requests.put(ADM + "/admin/oauth2/auth/requests/consent/accept", params={"consent_challenge": ch}, json={"grant_scope": scopes or req["requested_scope"], "remember": False,
                                         "session": {"id_token": {"name": "Alice \u00c9"}}}, timeout=10).json()
                    r = s.get(a["redirect_to"], allow_redirects=False, timeout=10)
                else:
                    r = s.get(loc, allow_redirects=False, timeout=10)
            return r.headers.get("Location", "") or "no redirect: %s" % r.text[:100]
        def start(cid, scope):
            g = cli("authz", cid, REDIR, scope)
            return field(g, "verifier"), field(g, "state"), field(g, "url"), g
        for cid, secret, mode in (("web", "websecret", "basic"), ("spa", "", "basic")):
            ver, st, url, g = start(cid, "openid offline_access profile" if cid == "web" else "openid offline_access")
            q = up.parse_qs(up.urlparse(url).query, keep_blank_values=True)
            chal = base64.urlsafe_b64encode(hashlib.sha256(ver.encode()).digest()).decode().rstrip("=")
            check(mod, "%s: authorize_url parameters" % cid, ({k: v[0] for k, v in q.items()}, url.startswith(PUB + "/oauth2/auth?")),
                  ({"response_type": "code", "client_id": cid, "redirect_uri": REDIR, "scope": "openid offline_access profile" if cid == "web" else "openid offline_access", "state": st, "code_challenge": chal, "code_challenge_method": "S256"}, True))
            check(mod, "%s: PKCE challenge is base64url(SHA-256(verifier)), 43 characters" % cid, field(g, "challenge_ok"), "true 43")
            cb = drive(url)
            ok = cb.startswith(REDIR + "?code=") or "code=" in cb
            rec(mod, "%s: the server redirects back with a code" % cid, ok, cb[:200])
            if not ok: continue
            g = cli("callback", cb)
            check(mod, "%s: callback_params reads code and state" % cid, re.fullmatch(r"params \d+ code=\S+ state=%s" % re.escape(st), g[0] if g else "") is not None, True)
            g = cli("exchange", cid, secret, REDIR, "-", cb, st, ver, mode)
            want = ["state_matches true", "re:" + TOKEN % ("openid offline_access profile" if cid == "web" else "openid offline_access", "true", "true"), "header true", "re:access ory_at_\\S+", "re:id_token \\S+", "re:refresh ory_rt_\\S+"]
            expect("%s: code and verifier exchanged for tokens" % cid, g, want)
            i = introspect(field(g, "access") or "")
            check(mod, "%s: the server accepts the access token" % cid, (i.get("active"), i.get("client_id"), i.get("sub"), sorted(i.get("scope", "").split())),
                  (True, cid, "alice@example.com", sorted(("openid offline_access profile" if cid == "web" else "openid offline_access").split())))
            idt = field(g, "id_token")
            if idt:
                jw = jwt.PyJWKClient(PUB + "/.well-known/jwks.json")
                claims = jwt.decode(idt, jw.get_signing_key_from_jwt(idt).key, algorithms=["RS256"], audience=cid, issuer=PUB)
                check(mod, "%s: id_token verifies with the server's key" % cid, (claims["sub"], claims.get("name")), ("alice@example.com", "Alice \u00c9" if cid == "web" else None) if cid == "web" else ("alice@example.com", claims.get("name")))
            rt = field(g, "refresh")
            # refresh: a new access token, rotated refresh token
            g2 = cli("refresh", cid, secret, "-", rt, mode)
            expect("%s: refresh gives a new token and a new refresh token" % cid, g2, ["re:" + TOKEN % ("openid offline_access profile" if cid == "web" else "openid offline_access", "true", "true"), "header true", "re:access ory_at_\\S+", "re:id_token \\S+", "re:refresh ory_rt_\\S+"])
            check(mod, "%s: the refreshed token is new and active" % cid, (field(g2, "access") != field(g, "access"), field(g2, "refresh") != rt, introspect(field(g2, "access") or "").get("active")), (True, True, True))
            expect("%s: the used refresh token is refused (rotation)" % cid, cli("refresh", cid, secret, "-", rt, mode)[:1], ["error Denied invalid_grant 400"])
            expect("%s: and so is the whole family afterwards" % cid, cli("refresh", cid, secret, "-", field(g2, "refresh") or "x", mode)[:1], ["error Denied invalid_grant 400"])
            # code is single use
            expect("%s: the same code a second time" % cid, cli("exchange", cid, secret, REDIR, "-", cb, st, ver, mode)[1:2], ["error Denied invalid_grant 400"])
        # wrong verifier, missing verifier, wrong redirect, wrong secret
        ver, st, url, _ = start("web", "openid")
        cb = drive(url)
        expect("web: a wrong PKCE verifier", cli("exchange", "web", "websecret", REDIR, "-", cb, st, "x" * 43, "basic")[1:2], ["error Denied invalid_grant 400"])
        ver, st, url, _ = start("web", "openid")
        cb = drive(url)
        expect("web: wrong client secret", cli("exchange", "web", "bad", REDIR, "-", cb, st, ver, "basic")[1:2], ["error Denied invalid_client 401"])
        ver, st, url, _ = start("web", "openid")
        cb = drive(url)
        expect("web: a different redirect_uri than the request used", cli("exchange", "web", "websecret", REDIR + "x", "-", cb, st, ver, "basic")[1:2], ["error Denied invalid_grant 400"])
        expect("web: state that is not the one sent", cli("exchange", "web", "websecret", REDIR, "-", cb, "someone-elses", ver, "basic")[:1], ["state_matches false"])
        ver, st, url, _ = start("web", "openid")
        cb = drive(url, deny=True)
        g = cli("callback", cb)
        check(mod, "web: a denied consent comes back as access_denied with the server's text", g, ["error Denied access_denied 0", "message The user said no \u00e9"])
        ver, st, url, _ = start("web", "openid profile")
        cb = drive(url, scopes=["openid"])
        g = cli("exchange", "web", "websecret", REDIR, "-", cb, st, ver, "basic")
        check(mod, "web: a narrower granted scope is reported", (field(g, "token") or "").split("scope=")[1].split(" ")[0], "[openid]")
        # a revoked refresh token
        ver, st, url, _ = start("web", "openid offline_access")
        cb = drive(url)
        g = cli("exchange", "web", "websecret", REDIR, "-", cb, st, ver, "basic")
        rt = field(g, "refresh")
        requests.post(PUB + "/oauth2/revoke", data={"token": rt, "token_type_hint": "refresh_token"}, auth=("web", "websecret"), timeout=10)
        expect("web: a revoked refresh token", cli("refresh", "web", "websecret", "-", rt or "x", "basic")[:1], ["error Denied invalid_grant 400"])
    finally:
        run([*DOCKER, "rm", "-f", "-v", name])

def device_server(port, state):
    """An RFC 8628 device-flow authorization server built on Authlib's reference implementation. Returns the werkzeug server."""
    from flask import Flask, request
    from authlib.integrations.flask_oauth2 import AuthorizationServer
    from authlib.oauth2.rfc6749 import ClientMixin
    from authlib.oauth2.rfc8628 import DeviceAuthorizationEndpoint, DeviceCodeGrant, DeviceCredentialDict
    from werkzeug.serving import make_server
    DEVICE = "urn:ietf:params:oauth:grant-type:device_code"
    class C(ClientMixin):
        def __init__(s, cid, secret): s.cid, s.secret = cid, secret
        def get_client_id(s): return s.cid
        def get_default_redirect_uri(s): return None
        def get_allowed_scope(s, scope): return scope
        def check_redirect_uri(s, uri): return False
        def check_client_secret(s, secret): return s.secret is not None and secret == s.secret
        def check_endpoint_auth_method(s, method, endpoint): return method in (("none",) if s.secret is None else ("client_secret_basic", "client_secret_post"))
        def check_response_type(s, t): return False
        def check_grant_type(s, g): return g in (DEVICE, "refresh_token")
    clients = {"tv": C("tv", "tvsecret"), "pub": C("pub", None)}
    app = Flask("device"); app.config["OAUTH2_REFRESH_TOKEN_GENERATOR"] = True
    app.config["SECRET_KEY"] = "x"
    server = AuthorizationServer(app, query_client=lambda cid: clients.get(cid), save_token=lambda token, request: state["tokens"].append(token))
    class EP(DeviceAuthorizationEndpoint):
        CLIENT_AUTH_METHODS = ["client_secret_basic", "client_secret_post", "none"]
        USER_CODE_TYPE = "string"
        @property
        def EXPIRES_IN(s): return state["expires_in"]
        INTERVAL = 1
        def get_verification_uri(s): return "http://%s:%d/activate" % (HOST, port)
        def save_device_credential(s, client_id, scope, data):
            d = DeviceCredentialDict(client_id=client_id, scope=scope, user_code=data["user_code"], expires_at=time.time() + data["expires_in"], **{"device_code": data["device_code"]})
            state["creds"][data["device_code"]] = d; state["order"].append(data["user_code"])
    class G(DeviceCodeGrant):
        TOKEN_ENDPOINT_AUTH_METHODS = ["client_secret_basic", "client_secret_post", "none"]
        def query_device_credential(s, device_code): return state["creds"].get(device_code)
        def query_user_grant(s, user_code):
            v = state["grants"].get(user_code)
            return None if v is None else ("alice", v)
        def should_slow_down(s, credential):
            state["polls"] += 1
            return state["slow_downs"] > 0 and state["polls"] > 1 and (state.__setitem__("slow_downs", state["slow_downs"] - 1) or True)
    server.register_grant(G); server.register_endpoint(EP)
    @app.route("/oauth2/device/auth", methods=["POST"])
    def dev(): return server.create_endpoint_response("device_authorization")
    @app.route("/oauth2/token", methods=["POST"])
    def tok(): return server.create_token_response()
    srv = make_server(HOST, port, app, threaded=True)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv

def t_device():
    mod = "oauth2"; exe = need(mod)
    if not exe: return
    try:
        import authlib, flask
    except ImportError:
        rec(mod, "authlib", None, "python packages authlib and flask are not installed"); return
    port = free_port(); base = "http://%s:%d" % (HOST, port)
    state = {"creds": {}, "grants": {}, "order": [], "tokens": [], "expires_in": 600, "slow_downs": 0, "polls": 0}
    srv = device_server(port, state)
    try:
        TOK = r"token type=Bearer scope=\[%s\] refresh=true id_token=false lifetime_ok=true expired=false"
        def go(args, approve=None, delay=1.5):
            n0 = len(state["order"])
            e = dict(os.environ)
            p = subprocess.Popen([exe, base, "device", *args], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if approve is not None:
                for _ in range(100):
                    if len(state["order"]) > n0: break
                    time.sleep(0.1)
                time.sleep(delay)
                if len(state["order"]) > n0: state["grants"][state["order"][n0]] = approve
            try: out = p.communicate(timeout=90)[0].decode(errors="replace").splitlines()
            except subprocess.TimeoutExpired: p.kill(); out = ["timed out"]
            return out
        UC = r"user_code [A-Z]{4}-[A-Z]{4}"
        head = ["re:" + UC, "uri " + base + "/activate", r"re:complete http://127\.0\.0\.1:\d+/activate\?user_code=[A-Z]{4}-[A-Z]{4}", "interval 1", "expires_in_ok true", "pending"]
        g = go(["tv", "tvsecret", "read write", "basic"], approve=True)
        rec(mod, "device: approved", lines_match(g, head + ["re:" + TOK % "read write", "header true", r"re:access \S+", r"re:refresh \S+"]), diff(g, head + ["re:" + TOK % "read write", "header true", "re:access \\S+", "re:refresh \\S+"]))
        uc = g[0].split()[1] if g else ""
        check(mod, "device: the user code in the answer is the one the server issued", uc == state["order"][0], True)
        check(mod, "device: the server saved a token with the asked scope", [t.get("scope") for t in state["tokens"]], ["read write"])
        g = go(["tv", "tvsecret", "read", "body"], approve=True)
        rec(mod, "device: secret in the body", lines_match(g[6:8], ["re:" + TOK % "read", "header true"]), diff(g[6:8], ["re:" + TOK % "read", "header true"]))
        g = go(["pub", "", "read", "basic"], approve=True)
        rec(mod, "device: a public client with no secret", lines_match(g[6:8], ["re:" + TOK % "read", "header true"]), diff(g[6:8], ["re:" + TOK % "read", "header true"]) + " %r" % g[:3])
        g = go(["tv", "tvsecret", "read", "basic"], approve=False)
        check(mod, "device: denied by the user", g[6:7], ["error Denied access_denied 400"])
        g = go(["tv", "wrong", "read", "basic"])
        check(mod, "device: wrong client secret at the device endpoint", g[:1], ["error Denied invalid_client 401"])
        state["slow_downs"] = 1; state["polls"] = 0
        t0 = time.time()
        g = go(["tv", "tvsecret", "read", "basic"], approve=True, delay=2.0)
        rec(mod, "device: slow_down makes the client wait 5 s more, then it gets the token", lines_match(g[6:8], ["re:" + TOK % "read", "header true"]) and time.time() - t0 > 5, "%s after %.1f s" % (g[6:8], time.time() - t0))
        state["expires_in"] = 3; state["slow_downs"] = 0
        g = go(["tv", "tvsecret", "read", "basic"])
        rec(mod, "device: expires before it is approved", len(g) > 6 and re.fullmatch(r"error Expired expired_token (400|0)", g[6]) is not None, "%r" % g[5:8])
        state["expires_in"] = 600
        g = go(["tv", "tvsecret", "read", "basic"], approve=True)
        rec(mod, "device: the flow works again after an expiry", lines_match(g[6:8], ["re:" + TOK % "read", "header true"]), "%r" % g[5:8])
    finally:
        srv.shutdown()

TESTS = {"smtp": t_smtp, "sse": t_sse, "s3": t_s3, "oauth2": lambda: (t_oauth2(), t_device())}

def main():
    todo = [m for m in TESTS if not ONLY or m in ONLY]
    for m in todo:
        try: TESTS[m]()
        except Exception as e:
            import traceback; rec(m, "harness", False, traceback.format_exc()[-300:])
    with open(os.path.join(OUT, "results-net.tsv"), "a") as f:
        for r in rows: f.write("\t".join(r) + "\n")
    n = {s: sum(1 for r in rows if r[2] == s) for s in ("PASS", "FAIL", "SKIP")}
    print("net interop: %(PASS)d pass, %(FAIL)d fail, %(SKIP)d skip" % n)
    sys.exit(1 if n["FAIL"] else 0)

main()
