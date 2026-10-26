"""Stands in for a program's attach socket in gate.ail's `net` suite (attach.cli.1): answers every request line
with a fixed reply and saves the line. Usage: python3 -I tools/gate/attach_server.py PORT REQUEST_FILE"""
import socket
import sys

s = socket.socket()
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(("127.0.0.1", int(sys.argv[1])))
s.listen(16)
while True:
    c, _ = s.accept()
    try:
        line = c.makefile("rb").readline()
        if line:
            open(sys.argv[2], "wb").write(line)
            c.sendall(b'{"v":1,"rev":1,"values":{}}\n')
    except OSError:
        pass  # a readiness probe connects and closes
    finally:
        c.close()
