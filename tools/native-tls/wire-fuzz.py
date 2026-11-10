"""TLS server that answers every client with a mutated copy of a recorded server flight,
for tools/native-tls/wire-fuzz.sh. The client under test must reject each one with an
error, never crash, hang or accept.
Usage: python3 -I tools/native-tls/wire-fuzz.py PORT SEED FLIGHT_FILE"""
import os
import random
import socket
import sys
import threading

# A server the harness could not stop must not outlive the run.
_guard = threading.Timer(900, lambda: os._exit(0))
_guard.daemon = True
_guard.start()

port, seed, path = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
lines = [l for l in open(path).read().splitlines() if l and not l.startswith("#")]
flight = bytes.fromhex(lines[1])  # the server's side of the recorded handshake


def mutate(rng, data):
    data = bytearray(data)
    for _ in range(rng.choice([1, 1, 1, 2, 3])):
        kind = rng.randrange(8)
        # Half of the edits land in the first 200 bytes: record headers and the ServerHello.
        top = len(data) if rng.random() < 0.5 else min(len(data), 200)
        if not data:
            data = bytearray(rng.randbytes(rng.randrange(1, 12)))
            continue
        i = rng.randrange(max(top, 1))
        if kind == 0:
            data[i] ^= 1 << rng.randrange(8)
        elif kind == 1:
            data[i] = rng.randrange(256)
        elif kind == 2:
            del data[i : i + rng.randrange(1, 9)]
        elif kind == 3:
            data[i:i] = rng.randbytes(rng.randrange(1, 9))
        elif kind == 4:
            del data[i:]
        elif kind == 5:
            data[i : i + 2] = bytes([rng.choice([0, 255]), rng.choice([0, 255])])
        elif kind == 6:
            j = min(len(data), i + rng.randrange(1, 40))
            data[j:j] = data[i:j]
        else:
            data = bytearray(rng.randbytes(rng.randrange(1, 64)))
    return bytes(data)


srv = socket.socket()
srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
srv.bind(("127.0.0.1", port))
srv.listen(16)
n = 0
while True:
    conn, _ = srv.accept()
    n += 1
    rng = random.Random(seed * 100003 + n)
    try:
        conn.settimeout(3)
        try:
            conn.recv(4096)  # the ClientHello
        except OSError:
            pass
        conn.sendall(mutate(rng, flight))
    except OSError:
        pass
    finally:
        conn.close()
