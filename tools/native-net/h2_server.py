#!/usr/bin/env python3
"""A scripted HTTP/2 server for tools/native-net/h2-check.sh (ADR 0913): TLS 1.3
with ALPN on 127.0.0.1, certificate for `localhost`. It prints its port on its
first line. Modes (the second argument): `h2` (ALPN h2, the default), `h1`
(ALPN http/1.1 only) and `none` (no ALPN extension); in the last two it
answers HTTP/1.1. Usage: h2_server.py CERT_CHAIN KEY [MODE [PORT]]. Python 3.8+ with
OpenSSL 1.1.1 or later (TLS 1.3), standard library only. It checks nothing
itself: the expected output of h2_client.ail (h2_client.out) is the test."""
import gzip
import socket
import ssl
import struct
import sys
import threading
import time

STATIC = [
    ("", ""),
    (':authority', ''),
    (':method', 'GET'),
    (':method', 'POST'),
    (':path', '/'),
    (':path', '/index.html'),
    (':scheme', 'http'),
    (':scheme', 'https'),
    (':status', '200'),
    (':status', '204'),
    (':status', '206'),
    (':status', '304'),
    (':status', '400'),
    (':status', '404'),
    (':status', '500'),
    ('accept-charset', ''),
    ('accept-encoding', 'gzip, deflate'),
    ('accept-language', ''),
    ('accept-ranges', ''),
    ('accept', ''),
    ('access-control-allow-origin', ''),
    ('age', ''),
    ('allow', ''),
    ('authorization', ''),
    ('cache-control', ''),
    ('content-disposition', ''),
    ('content-encoding', ''),
    ('content-language', ''),
    ('content-length', ''),
    ('content-location', ''),
    ('content-range', ''),
    ('content-type', ''),
    ('cookie', ''),
    ('date', ''),
    ('etag', ''),
    ('expect', ''),
    ('expires', ''),
    ('from', ''),
    ('host', ''),
    ('if-match', ''),
    ('if-modified-since', ''),
    ('if-none-match', ''),
    ('if-range', ''),
    ('if-unmodified-since', ''),
    ('last-modified', ''),
    ('link', ''),
    ('location', ''),
    ('max-forwards', ''),
    ('proxy-authenticate', ''),
    ('proxy-authorization', ''),
    ('range', ''),
    ('referer', ''),
    ('refresh', ''),
    ('retry-after', ''),
    ('server', ''),
    ('set-cookie', ''),
    ('strict-transport-security', ''),
    ('transfer-encoding', ''),
    ('user-agent', ''),
    ('vary', ''),
    ('via', ''),
    ('www-authenticate', ''),
]
HUFF = (
    "00001ff80d007fffd8170fffffe21c0fffffe31c0fffffe41c0fffffe51c0fffffe61c0fffffe71c0fffffe81c00ffffea18"
    "3ffffffc1e0fffffe91c0fffffea1c3ffffffd1e0fffffeb1c0fffffec1c0fffffed1c0fffffee1c0fffffef1c0ffffff01c"
    "0ffffff11c0ffffff21c3ffffffe1e0ffffff31c0ffffff41c0ffffff51c0ffffff61c0ffffff71c0ffffff81c0ffffff91c"
    "0ffffffa1c0ffffffb1c0000001406000003f80a000003f90a00000ffa0c00001ff90d0000001506000000f808000007fa0b"
    "000003fa0a000003fb0a000000f908000007fb0b000000fa0800000016060000001706000000180600000000050000000105"
    "000000020500000019060000001a060000001b060000001c060000001d060000001e060000001f060000005c07000000fb08"
    "00007ffc0f000000200600000ffb0c000003fc0a00001ffa0d00000021060000005d070000005e070000005f070000006007"
    "0000006107000000620700000063070000006407000000650700000066070000006707000000680700000069070000006a07"
    "0000006b070000006c070000006d070000006e070000006f07000000700700000071070000007207000000fc080000007307"
    "000000fd0800001ffb0d0007fff01300001ffc0d00003ffc0e000000220600007ffd0f000000030500000023060000000405"
    "0000002406000000050500000025060000002606000000270600000006050000007407000000750700000028060000002906"
    "0000002a0600000007050000002b0600000076070000002c06000000080500000009050000002d0600000077070000007807"
    "00000079070000007a070000007b0700007ffe0f000007fc0b00003ffd0e00001ffd0d0ffffffc1c000fffe614003fffd216"
    "000fffe714000fffe814003fffd316003fffd416003fffd516007fffd917003fffd616007fffda17007fffdb17007fffdc17"
    "007fffdd17007fffde1700ffffeb18007fffdf1700ffffec1800ffffed18003fffd716007fffe01700ffffee18007fffe117"
    "007fffe217007fffe317007fffe417001fffdc15003fffd816007fffe517003fffd916007fffe617007fffe71700ffffef18"
    "003fffda16001fffdd15000fffe914003fffdb16003fffdc16007fffe817007fffe917001fffde15007fffea17003fffdd16"
    "003fffde1600fffff018001fffdf15003fffdf16007fffeb17007fffec17001fffe015001fffe115003fffe016001fffe215"
    "007fffed17003fffe116007fffee17007fffef17000fffea14003fffe216003fffe316003fffe416007ffff017003fffe516"
    "003fffe616007ffff11703ffffe01a03ffffe11a000fffeb140007fff113003fffe716007ffff217003fffe81601ffffec19"
    "03ffffe21a03ffffe31a03ffffe41a07ffffde1b07ffffdf1b03ffffe51a00fffff11801ffffed190007fff213001fffe315"
    "03ffffe61a07ffffe01b07ffffe11b03ffffe71a07ffffe21b00fffff218001fffe415001fffe51503ffffe81a03ffffe91a"
    "0ffffffd1c07ffffe31b07ffffe41b07ffffe51b000fffec1400fffff318000fffed14001fffe615003fffe916001fffe715"
    "001fffe815007ffff317003fffea16003fffeb1601ffffee1901ffffef1900fffff41800fffff51803ffffea1a007ffff417"
    "03ffffeb1a07ffffe61b03ffffec1a03ffffed1a07ffffe71b07ffffe81b07ffffe91b07ffffea1b07ffffeb1b0ffffffe1c"
    "07ffffec1b07ffffed1b07ffffee1b07ffffef1b07fffff01b03ffffee1a3fffffff1e"
)


PORT = 0
CONNS = 0
HUFF_CODES = [(int(HUFF[i * 10:i * 10 + 8], 16), int(HUFF[i * 10 + 8:i * 10 + 10], 16)) for i in range(257)]


def huff_encode(data):
    bits = 0
    nbits = 0
    out = bytearray()
    for b in data:
        code, ln = HUFF_CODES[b]
        bits = (bits << ln) | code
        nbits += ln
        while nbits >= 8:
            nbits -= 8
            out.append((bits >> nbits) & 255)
        bits &= (1 << nbits) - 1
    if nbits:
        out.append(((bits << (8 - nbits)) | ((1 << (8 - nbits)) - 1)) & 255)
    return bytes(out)


def int_enc(v, bits, first):
    mx = (1 << bits) - 1
    if v < mx:
        return bytes([first | v])
    out = bytearray([first | mx])
    v -= mx
    while v >= 128:
        out.append((v & 127) | 128)
        v >>= 7
    out.append(v)
    return bytes(out)


def str_enc(s, huff=False):
    s = s.encode() if isinstance(s, str) else s
    if huff:
        h = huff_encode(s)
        return int_enc(len(h), 7, 128) + h
    return int_enc(len(s), 7, 0) + s


def f_indexed(i):
    return int_enc(i, 7, 128)


def f_lit(name, value, huff=False, incr=False):
    """A literal field; `name` is a table index or a string."""
    first = 64 if incr else 0
    bits = 6 if incr else 4
    if isinstance(name, int):
        return int_enc(name, bits, first) + str_enc(value, huff)
    return int_enc(0, bits, first) + str_enc(name, huff) + str_enc(value, huff)


def int_dec(buf, i, bits):
    mx = (1 << bits) - 1
    v = buf[i] & mx
    i += 1
    if v < mx:
        return v, i
    m = 0
    while True:
        b = buf[i]
        i += 1
        v += (b & 127) << m
        m += 7
        if b < 128:
            return v, i


def huff_decode(data):
    table = {(ln, code): sym for sym, (code, ln) in enumerate(HUFF_CODES)}
    out = bytearray()
    code = ln = 0
    for byte in data:
        for k in range(7, -1, -1):
            code = (code << 1) | ((byte >> k) & 1)
            ln += 1
            sym = table.get((ln, code))
            if sym is not None:
                out.append(sym)
                code = ln = 0
    return bytes(out)


def str_dec(buf, i):
    huff = buf[i] & 128
    n, i = int_dec(buf, i, 7)
    raw = buf[i:i + n]
    return (huff_decode(raw) if huff else raw).decode("latin-1"), i + n


class Decoder:
    def __init__(self):
        self.dyn = []

    def entry(self, i):
        if i < len(STATIC):
            return STATIC[i]
        return self.dyn[i - len(STATIC)]

    def decode(self, buf):
        out = []
        i = 0
        while i < len(buf):
            b = buf[i]
            if b & 128:
                idx, i = int_dec(buf, i, 7)
                out.append(self.entry(idx))
            elif b & 224 == 32:
                _, i = int_dec(buf, i, 5)
            else:
                incr = b & 192 == 64
                idx, i = int_dec(buf, i, 6 if incr else 4)
                if idx:
                    name = self.entry(idx)[0]
                else:
                    name, i = str_dec(buf, i)
                value, i = str_dec(buf, i)
                if incr:
                    self.dyn.insert(0, (name, value))
                out.append((name, value))
        return out


def frame(ty, flags, stream, payload=b""):
    return struct.pack(">I", len(payload))[1:] + bytes([ty, flags]) + struct.pack(">I", stream) + payload


def recv_exact(sock, n):
    data = b""
    while len(data) < n:
        chunk = sock.recv(n - len(data))
        if not chunk:
            raise EOFError
        data += chunk
    return data


PATTERN = "0123456789abcdef"


class Conn:
    def __init__(self, sock):
        self.sock = sock
        self.dec = Decoder()
        self.cwin = 65535
        self.swin = 65535
        self.iwin = 65535
        self.ping_acked = False
        global CONNS
        CONNS += 1
        self.number = CONNS
        self.requests = 0

    def send(self, data):
        self.sock.sendall(data)

    def next_frame(self, once=False):
        """The next frame that is not connection housekeeping (with `once`,
        after any one frame: None for housekeeping)."""
        while True:
            h = recv_exact(self.sock, 9)
            n = int.from_bytes(h[:3], "big")
            ty, fl = h[3], h[4]
            sid = int.from_bytes(h[5:9], "big") & 0x7FFFFFFF
            payload = recv_exact(self.sock, n) if n else b""
            if ty == 4:
                if not fl & 1:
                    for k in range(0, n, 6):
                        if int.from_bytes(payload[k:k + 2], "big") == 4:
                            v = int.from_bytes(payload[k + 2:k + 6], "big")
                            self.swin += v - self.iwin
                            self.iwin = v
                    self.send(frame(4, 1, 0))
                if once:
                    return None
            elif ty == 6:
                if fl & 1:
                    self.ping_acked = True
                else:
                    self.send(frame(6, 1, 0, payload))
                if once:
                    return None
            elif ty == 8:
                inc = int.from_bytes(payload, "big") & 0x7FFFFFFF
                if sid == 0:
                    self.cwin += inc
                else:
                    self.swin += inc
                if once:
                    return None
            else:
                return ty, fl, sid, payload

    def send_data(self, sid, data, last=True, pad=0):
        pos = 0
        while pos < len(data) or (pos == 0 and last and not data):
            k = min(16384 - (pad + 1 if pad else 0), len(data) - pos, self.cwin, self.swin)
            if k <= 0 and pos < len(data):
                self.next_frame(once=True)
                continue
            end = pos + k >= len(data)
            fl = 1 if (end and last) else 0
            chunk = data[pos:pos + k]
            if pad:
                fl |= 8
                chunk = bytes([pad]) + chunk + bytes(pad)
            self.send(frame(0, fl, sid, chunk))
            self.cwin -= k
            self.swin -= k
            pos += k
            if end:
                break


def status_field(code):
    return {200: f_indexed(8), 204: f_indexed(9), 404: f_indexed(13)}.get(code) or f_lit(8, str(code))


def serve_h2(conn, tls):
    c = Conn(tls)
    if recv_exact(tls, 24) != b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n":
        return
    c.send(frame(4, 0, 0))
    try:
        serve_streams(c, tls)
    except (EOFError, OSError):
        pass


def serve_streams(c, tls):
    block = b""
    method = path = ""
    headers = []
    body = b""
    sid = 0
    while True:
        ty, fl, sid1, p = c.next_frame()
        if ty == 1 or ty == 9:
            if ty == 1:
                sid = sid1
                c.swin = c.iwin
                off = 0
                if fl & 8:
                    off = 1
                    p = p[:len(p) - p[0]]
                if fl & 32:
                    off += 5
                p = p[off:]
                block = b""
            block += p
            if fl & 4:
                headers = c.dec.decode(block)
                d = dict(headers)
                method = d.get(":method", "")
                path = d.get(":path", "")
            if ty == 1 and fl & 1:
                respond(c, sid, method, path, headers, body)
                body = b""
        elif ty == 0:
            if fl & 8:
                p = p[1:len(p) - p[0]]
            body += p
            if p:
                c.send(frame(8, 0, 0, struct.pack(">I", len(p))) + frame(8, 0, sid, struct.pack(">I", len(p))))
            if fl & 1:
                respond(c, sid, method, path, headers, body)
                body = b""


def respond(c, sid, method, path, headers, body):
    head = method == "HEAD"
    ctype = f_lit(31, "text/plain")

    def reply(code, fields, data, extra=b""):
        block = status_field(code) + fields
        if head or not data:
            c.send(frame(1, 4 | 1, sid, block))
            return
        c.send(frame(1, 4, sid, block))
        c.send_data(sid, data)

    c.requests += 1
    if path == "/which":
        reply(200, ctype, ("conn %d request %d\n" % (c.number, c.requests)).encode())
    elif path == "/hello":
        reply(200, ctype + f_lit("x-proto", "h2") + f_lit(55, "a=1") + f_lit(55, "b=2")
              + f_lit(28, "14" if not head else "14"), b"hello over h2\n")
    elif path == "/hdrs":
        lines = []
        for n, v in headers:
            if n == ":authority" and v == "localhost:%d" % PORT:
                v = "localhost:PORT"
            lines.append("%s: %s\n" % (n, v))
        reply(200, ctype, "".join(lines).encode())
    elif path == "/big":
        reply(200, ctype, (PATTERN * 18750).encode())
    elif path == "/huge":
        reply(200, ctype, (PATTERN * (8 * 1024 * 1024 // 16)).encode())
    elif path == "/post":
        want = (PATTERN * (len(body) // 16 + 1))[:len(body)].encode()
        reply(200, ctype, ("got %d bytes, %s\n" % (len(body), "same" if body == want else "different")).encode())
    elif path == "/redirect":
        reply(302, f_lit(46, "/hello"), b"")
    elif path == "/nobody":
        reply(204, b"", b"")
    elif path == "/missing":
        reply(404, ctype, b"nope")
    elif path == "/gzip":
        reply(200, ctype + f_lit(26, "gzip"), gzip.compress(b"zipped over h2\n"))
    elif path == "/interim":
        c.send(frame(1, 4, sid, f_lit(8, "103") + f_lit("link", "</style.css>; rel=preload")))
        reply(200, ctype + f_lit("x-after", "interim"), b"after interim\n")
    elif path == "/trailers":
        c.send(frame(1, 4, sid, status_field(200) + ctype))
        c.send_data(sid, b"with trailers\n", last=False)
        c.send(frame(1, 4 | 1, sid, f_lit("x-trailer", "dropped")))
    elif path == "/huffman":
        block = (int_enc(4096, 5, 32) + status_field(200)
                 + f_lit("x-dyn", "dynamic value", huff=True, incr=True)
                 + f_indexed(62)
                 + f_lit("x-huff", "www.example.com", huff=True)
                 + f_lit(62, "second", huff=True, incr=True)
                 + f_lit(31, "text/plain", huff=True))
        c.send(frame(1, 4, sid, block))
        c.send_data(sid, b"huffman ok\n")
    elif path == "/split":
        block = status_field(200) + ctype + f_lit("x-split", "a" * 40) + f_lit("x-more", "b" * 40)
        c.send(frame(1, 0, sid, block[:20]))
        c.send(frame(9, 0, sid, block[20:60]))
        c.send(frame(9, 4, sid, block[60:]))
        c.send_data(sid, b"split and padded\n", pad=7)
    elif path == "/ping":
        c.ping_acked = False
        c.send(frame(6, 0, 0, b"12345678"))
        t = time.time()
        c.tls_timeout = c.sock.gettimeout()
        c.sock.settimeout(0.2)
        while not c.ping_acked and time.time() - t < 3:
            try:
                c.next_frame()
            except socket.timeout:
                pass
        c.sock.settimeout(c.tls_timeout)
        reply(200, ctype + f_lit("x-ping", "acked" if c.ping_acked else "missing"), b"pinged\n")
    elif path == "/goaway":
        c.send(frame(7, 0, 0, struct.pack(">II", 0, 0)))
    elif path == "/rst":
        c.send(frame(3, 0, sid, struct.pack(">I", 2)))
    else:
        reply(404, ctype, b"unknown path")


def serve_h1(tls):
    data = b""
    while b"\r\n\r\n" not in data:
        chunk = tls.recv(4096)
        if not chunk:
            return
        data += chunk
    body = b"fallback http/1.1\n"
    tls.sendall(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: %d\r\nConnection: close\r\n\r\n" % len(body) + body)


def handle(sock, ctx):
    try:
        tls = ctx.wrap_socket(sock, server_side=True)
        tls.settimeout(10)
        if tls.selected_alpn_protocol() == "h2":
            serve_h2(sock, tls)
        else:
            serve_h1(tls)
        tls.close()
    except Exception as e:
        print("server error: %r" % (e,), file=sys.stderr, flush=True)
    finally:
        sock.close()


def main():
    global PORT
    chain, key = sys.argv[1], sys.argv[2]
    mode = sys.argv[3] if len(sys.argv) > 3 else "h2"
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.minimum_version = ssl.TLSVersion.TLSv1_3
    ctx.load_cert_chain(chain, key)
    if mode == "h2":
        ctx.set_alpn_protocols(["h2", "http/1.1"])
    elif mode == "h1":
        ctx.set_alpn_protocols(["http/1.1"])
    srv = socket.socket()
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", int(sys.argv[4]) if len(sys.argv) > 4 else 0))
    srv.listen(16)
    PORT = srv.getsockname()[1]
    print(PORT, flush=True)
    while True:
        s, _ = srv.accept()
        threading.Thread(target=handle, args=(s, ctx), daemon=True).start()


main()
