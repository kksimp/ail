"""Loopback HTTP server for gate.ail's `net` suite: the routes bootstrap/tests/net/client.ail asks for.
Usage: python3 -I tools/gate/net_server.py PORT"""
import socket
import sys


def serve(conn):
    f = conn.makefile("rb")
    method, path = f.readline().decode().split()[:2]
    length, content_type = 0, ""
    while True:
        h = f.readline().decode().rstrip("\r\n")
        if not h:
            break
        name, value = h.split(":", 1)
        if name.lower() == "content-length":
            length = int(value.strip())
        elif name.lower() == "content-type":
            content_type = value.strip()
    body = f.read(length)
    if path == "/hello":
        reply = "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nX-Twice: 1\r\nX-Twice: 2\r\nContent-Length: 12\r\n\r\nhello, agent"
    elif path == "/moved":
        reply = "HTTP/1.1 302 Found\r\nLocation: /hello\r\nContent-Length: 0\r\n\r\n"
    elif path == "/chunked":
        reply = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n"
    elif path == "/echo":
        text = f"{method} {path} {content_type} {length} {body.decode(errors='replace')}"
        reply = f"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{text}"
    else:
        reply = "HTTP/1.1 404 Not Found\r\nContent-Length: 12\r\n\r\nno such page"
    conn.sendall(reply.encode())


s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(sys.argv[1])))
s.listen(16)
while True:
    c, _ = s.accept()
    try:
        serve(c)
    except (ValueError, OSError):
        pass  # a readiness probe connects and closes
    finally:
        c.close()
