#!/usr/bin/env python3
"""A TLS front for a database server on a unix socket, for the interop checks
(tools/interop/db-tls.sh) where the server's own TLS cannot talk to AIL's TLS
client (it offers X25519 only; PostgreSQL 16 and MySQL 8.0 accept only NIST
groups). It speaks the real in-band upgrade of each protocol to the client,
terminates TLS with Python's OpenSSL, and relays plaintext to the server's unix
socket. usage: tls_front.py pg|my LISTEN_PORT UNIX_SOCKET CERT KEY
"""
import socket, ssl, struct, sys, threading

kind, port, sock_path, cert, key = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4], sys.argv[5]
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.minimum_version = ssl.TLSVersion.TLSv1_3
ctx.load_cert_chain(cert, key)

def recv_n(s, n):
    buf = b""
    while len(buf) < n:
        d = s.recv(n - len(buf))
        if not d:
            raise EOFError
        buf += d
    return buf

def pump(a, b):
    try:
        while True:
            d = a.recv(65536)
            if not d:
                break
            b.sendall(d)
    except (OSError, EOFError):
        pass
    for s in (a, b):
        try: s.shutdown(socket.SHUT_RDWR)
        except OSError: pass

def relay(c, srv):
    t = threading.Thread(target=pump, args=(srv, c), daemon=True); t.start()
    pump(c, srv)

def backend():
    s = socket.socket(socket.AF_UNIX); s.connect(sock_path); return s

def handle_pg(c):
    first = recv_n(c, 8)
    srv = backend()
    if first == struct.pack(">II", 8, 80877103):
        c.sendall(b"S")
        c = ctx.wrap_socket(c, server_side=True)
        print("pg: TLS accepted", c.version(), c.cipher()[0], flush=True)
    else:
        srv.sendall(first)
        print("pg: plain", flush=True)
    relay(c, srv)

def packet(s):
    head = recv_n(s, 4)
    n = head[0] | head[1] << 8 | head[2] << 16
    return head[3], recv_n(s, n)

def send_packet(s, seq, body):
    s.sendall(struct.pack("<I", len(body))[:3] + bytes([seq]) + body)

def handle_my(c):
    srv = backend()
    seq, hello = packet(srv)
    send_packet(c, seq, hello)
    seq, body = packet(c)
    caps = struct.unpack("<I", body[:4])[0]
    if len(body) == 32 and caps & 0x800:
        c = ctx.wrap_socket(c, server_side=True)
        print("my: TLS accepted", c.version(), c.cipher()[0], flush=True)
        seq, body = packet(c)
        caps = struct.unpack("<I", body[:4])[0]
        if not caps & 0x800:
            print("my: login after TLS lacks CLIENT_SSL", flush=True)
        body = struct.pack("<I", caps & ~0x800) + body[4:]
        send_packet(srv, 1, body)
    else:
        print("my: plain", flush=True)
        send_packet(srv, seq, body)
    relay(c, srv)

handler = handle_pg if kind == "pg" else handle_my
ls = socket.socket(); ls.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
ls.bind(("127.0.0.1", port)); ls.listen(16)
print("listening", port, flush=True)
while True:
    c, _ = ls.accept()
    def run(c=c):
        try: handler(c)
        except Exception as e: print(kind, "error", repr(e), flush=True)
    threading.Thread(target=run, daemon=True).start()
