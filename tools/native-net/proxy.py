#!/usr/bin/env python3
"""A scripted HTTP proxy for proxy-check.sh. Prints its port, then answers
each connection with what it saw: the request line and the Host and
Proxy-Authorization headers. A CONNECT is refused with 403."""
import socket
import sys

srv = socket.socket()
srv.bind(("127.0.0.1", 0))
srv.listen(8)
print(srv.getsockname()[1], flush=True)
for _ in range(int(sys.argv[1])):
    c, _ = srv.accept()
    data = b""
    while b"\r\n\r\n" not in data:
        chunk = c.recv(4096)
        if not chunk:
            break
        data += chunk
    lines = data.decode("latin-1").split("\r\n")
    req = lines[0]
    hdr = {k.lower(): v for k, v in (l.split(": ", 1) for l in lines[1:] if ": " in l)}
    if req.startswith("CONNECT"):
        body, status = b"no tunnels", b"403 Forbidden"
    else:
        body = "{} host={} auth={}".format(
            req.rsplit(" ", 1)[0], hdr.get("host", "-"), hdr.get("proxy-authorization", "-")
        ).encode()
        status = b"200 OK"
    c.sendall(b"HTTP/1.1 " + status + b"\r\nContent-Length: " + str(len(body)).encode() + b"\r\nConnection: close\r\n\r\n" + body)
    c.close()
