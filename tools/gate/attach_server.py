"""Stands in for a program's attach socket in gate.ail's `net` suite (attach.cli.1): answers every request line
with a fixed reply and saves the line. Usage: python3 -I tools/gate/attach_server.py PORT REQUEST_FILE"""
import socket
import threading
import os
import sys

# A server the gate could not stop (on Windows `kill` may not reach it) must not outlive the run: it would
# hold the gate's output pipe open and keep `ail selftest` from exiting. The net suite takes minutes.
_guard = threading.Timer(1200, lambda: os._exit(0))
_guard.daemon = True
_guard.start()

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
