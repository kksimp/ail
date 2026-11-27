"""Stands in for a program's attach socket in gate.ail's `net` suite (attach.cli.1): answers every request line
with a fixed reply and saves the line. Usage: python3 -I tools/gate/attach_server.py PORT REQUEST_FILE [REPLY_FILE]
(the reply is the first line of REPLY_FILE when given, as the agent_tools suite's `ail attach mcp` session needs a `tools` reply).
With `--exit` for REPLY_FILE it saves the first request and exits without a reply, as a program whose quit key ends it
(attach.input.4)."""
import socket
import threading
import os
import sys

# A server the gate could not stop (on Windows `kill` may not reach it) must not outlive the run: it would
# hold the gate's output pipe open and keep `ail selftest` from exiting. The net suite takes minutes.
_guard = threading.Timer(1200, lambda: os._exit(0))
_guard.daemon = True
_guard.start()

reply = b'{"v":1,"rev":1,"values":{}}\n'
exit_on_request = len(sys.argv) > 3 and sys.argv[3] == "--exit"
if len(sys.argv) > 3 and not exit_on_request:
    reply = open(sys.argv[3], "rb").readline().rstrip(b"\n") + b"\n"

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
            if exit_on_request:
                os._exit(0)
            c.sendall(reply)
    except OSError:
        pass  # a readiness probe connects and closes
    finally:
        c.close()
