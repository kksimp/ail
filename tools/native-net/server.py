#!/usr/bin/env python3
"""A scripted HTTP server for tools/native-net/check.sh: it prints its port on
its first line and serves canned responses on 127.0.0.1."""
import socket
import sys
import threading


def read_request(conn):
    data = b""
    while b"\r\n\r\n" not in data:
        chunk = conn.recv(4096)
        if not chunk:
            return None
        data += chunk
    head, _, rest = data.partition(b"\r\n\r\n")
    lines = head.decode("latin-1").split("\r\n")
    length = 0
    for line in lines[1:]:
        if line.lower().startswith("content-length:"):
            length = int(line.split(":", 1)[1])
    if any(l.lower().startswith("transfer-encoding:") and "chunked" in l.lower() for l in lines[1:]):
        while not rest.endswith(b"0\r\n\r\n"):
            more = conn.recv(4096)
            if not more:
                break
            rest += more
        body = b""
        while rest:
            size, _, rest = rest.partition(b"\r\n")
            n = int(size, 16)
            if n == 0:
                break
            body += rest[:n]
            rest = rest[n + 2:]
        return lines[0], lines[1:], body
    while len(rest) < length:
        rest += conn.recv(4096)
    return lines[0], lines[1:], rest[:length]


def respond(conn, method, target, headers, body):
    if target == "/plain":
        data = b"hello, plain"
        return b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: %d\r\n\r\n" % len(data) + data
    if target == "/chunked":
        parts = [b"chunk one, ", b"chunk two, ", b"and a third \xc3\xa9"]
        out = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nX-Trailer-Test: 1\r\n\r\n"
        for p in parts:
            out += b"%x\r\n" % len(p) + p + b"\r\n"
        return out + b"0\r\nX-Late: yes\r\n\r\n"
    if target == "/close":
        return b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\nbody until the end"
    if target == "/redirect":
        return b"HTTP/1.1 302 Found\r\nLocation: /plain\r\nContent-Length: 0\r\n\r\n"
    if target == "/redirect-relative":
        return b"HTTP/1.1 301 Moved\r\nLocation: plain\r\nContent-Length: 0\r\n\r\n"
    if target == "/loop":
        return b"HTTP/1.1 302 Found\r\nLocation: /loop\r\nContent-Length: 0\r\n\r\n"
    if target == "/missing":
        data = b"nothing here"
        return b"HTTP/1.1 404 Not Found\r\nContent-Length: %d\r\n\r\n" % len(data) + data
    if target == "/headers":
        return (
            b"HTTP/1.1 200 OK\r\nX-Multi: one\r\nx-multi: two\r\nSet-Cookie: a=1\r\n"
            b"Set-Cookie: b=2\r\nX-Spaced:   padded value  \r\nContent-Length: 2\r\n\r\nok"
        )
    if target == "/big":
        data = (b"0123456789" * 10000)
        return b"HTTP/1.1 200 OK\r\nContent-Length: %d\r\n\r\n" % len(data) + data
    if target == "/badutf8":
        data = b"ok \xff\xfe and \xe2\x82 cut"
        return b"HTTP/1.1 200 OK\r\nContent-Length: %d\r\n\r\n" % len(data) + data
    if target == "/nocontent":
        return b"HTTP/1.1 204 No Content\r\nX-A: b\r\n\r\n"
    if target == "/continue":
        return b"HTTP/1.1 100 Continue\r\n\r\nHTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi"
    if target == "/garbage":
        return b"not http at all\r\n\r\n"
    if target == "/truncated":
        return b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nshort"
    if target == "/xorigin":
        host = [h.split(":", 1)[1].strip() for h in headers if h.lower().startswith("host:")][0]
        port = host.rsplit(":", 1)[1]
        return b"HTTP/1.1 302 Found\r\nLocation: http://localhost:%s/hdrs\r\nContent-Length: 0\r\n\r\n" % port.encode()
    if target.startswith("/hdrs"):
        keep = ("authorization", "cookie", "x-a", "content-type", "content-length", "transfer-encoding", "user-agent", "accept")
        shown = sorted(h.lower() if h.split(":", 1)[0].lower() != "x-a" else h for h in headers if h.split(":", 1)[0].lower() in keep)
        data = ("%s %s\n%s\nbody=%s" % (method, target, "\n".join(shown), body.decode("utf-8", "replace"))).encode()
        return b"HTTP/1.1 200 OK\r\nContent-Length: %d\r\n\r\n" % len(data) + data
    if target.startswith("/echo"):
        ctype = ""
        for h in headers:
            if h.lower().startswith("content-type:"):
                ctype = h.split(":", 1)[1].strip()
        data = ("%s %s|%s|%s" % (method, target, ctype, body.decode("utf-8", "replace"))).encode()
        return b"HTTP/1.1 200 OK\r\nContent-Length: %d\r\n\r\n" % len(data) + data
    return b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n"


def serve(conn):
    try:
        req = read_request(conn)
        if req is None:
            return
        line, headers, body = req
        method, target, _ = line.split(" ", 2)
        conn.sendall(respond(conn, method, target, headers, body))
    finally:
        conn.close()


def main():
    srv = socket.socket()
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", 0))
    srv.listen(64)
    print(srv.getsockname()[1], flush=True)
    while True:
        conn, _ = srv.accept()
        threading.Thread(target=serve, args=(conn,), daemon=True).start()


if __name__ == "__main__":
    sys.exit(main())
