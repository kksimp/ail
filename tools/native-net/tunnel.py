#!/usr/bin/env python3
"""A scripted CONNECT proxy for proxy-tunnel-check.sh. Prints its port, then
accepts one connection: it reads the CONNECT line, prints it to stderr,
answers 200, connects to 127.0.0.1:PORT (argv[1]) whatever host was asked
for, and relays bytes both ways until either side closes."""
import socket
import sys
import threading

target = int(sys.argv[1])
srv = socket.socket()
srv.bind(("127.0.0.1", 0))
srv.listen(1)
print(srv.getsockname()[1], flush=True)
c, _ = srv.accept()
data = b""
while b"\r\n\r\n" not in data:
    chunk = c.recv(4096)
    if not chunk:
        break
    data += chunk
head, _, rest = data.partition(b"\r\n\r\n")
print(head.split(b"\r\n")[0].decode("latin-1"), file=sys.stderr, flush=True)
up = socket.create_connection(("127.0.0.1", target))
c.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
if rest:
    up.sendall(rest)


def pump(a, b):
    try:
        while True:
            d = a.recv(65536)
            if not d:
                break
            b.sendall(d)
    except OSError:
        pass
    for s in (a, b):
        try:
            s.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass


t = threading.Thread(target=pump, args=(up, c))
t.start()
pump(c, up)
t.join()
