"""Client half of ws-deflate-check.sh: the Python `websockets` client (which
offers permessage-deflate with context takeover) against the AIL server's
compressed replies, and the server's raw frames against zlib."""
import asyncio, random, socket, sys, zlib

import websockets


def messages():
    random.seed(7)
    words = ["alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta"]
    yield "hello"
    yield "hello"
    yield ""
    yield "hello hello hello hello " * 40
    for _ in range(30):
        yield " ".join(random.choice(words) for _ in range(random.randint(1, 400)))
    big = "".join(random.choice(words) + "\n" for _ in range(20000))   # over 32 KiB
    yield big
    yield big[::-1]
    yield big
    yield bytes(random.randrange(256) for _ in range(5000))
    yield bytes(random.randrange(4) for _ in range(70000))


async def interop(port):
    async with websockets.connect(f"ws://127.0.0.1:{port}/echo") as ws:
        names = [e.name for e in ws.protocol.extensions]
        assert names == ["permessage-deflate"], f"extension not agreed: {names}"
        n = 0
        for m in messages():
            await ws.send(m)
            got = await ws.recv()
            assert got == m, f"message {n} came back different"
            n += 1
    print(f"websockets client: {n} messages echoed through permessage-deflate")


def frame_payload(sock, buf):
    def need(k):
        while len(buf) < k:
            chunk = sock.recv(65536)
            assert chunk, "connection closed"
            buf.extend(chunk)
    need(2)
    b0, b1 = buf[0], buf[1] & 127
    off = 2
    if b1 == 126:
        need(4)
        b1 = int.from_bytes(buf[2:4], "big"); off = 4
    elif b1 == 127:
        need(10)
        b1 = int.from_bytes(buf[2:10], "big"); off = 10
    need(off + b1)
    payload = bytes(buf[off:off + b1])
    del buf[:off + b1]
    return b0, payload


def raw_zlib(port):
    """Masked client frames by hand; the server's compressed replies, one
    zlib stream with a sync flush after each, inflate with `zlib`."""
    import base64, os
    s = socket.create_connection(("127.0.0.1", port))
    key = base64.b64encode(os.urandom(16)).decode()
    s.sendall((f"GET /echo HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
               f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n"
               f"Sec-WebSocket-Extensions: permessage-deflate\r\n\r\n").encode())
    buf = bytearray()
    while b"\r\n\r\n" not in buf:
        buf.extend(s.recv(4096))
    head, _, rest = bytes(buf).partition(b"\r\n\r\n")
    buf = bytearray(rest)
    assert b"permessage-deflate" in head and b"server_no_context_takeover" not in head, head
    d = zlib.decompressobj(-15)
    sent = []
    for text in ["a" * 300, "b" * 200 + "a" * 300, "the same words " * 100, "the same words " * 100]:
        data = text.encode()
        mask = os.urandom(4)
        body = bytes(c ^ mask[i % 4] for i, c in enumerate(data))   # uncompressed frame, no RSV1
        n = len(body)
        hdr = bytes([0x81]) + (bytes([128 | n]) if n < 126 else bytes([128 | 126]) + n.to_bytes(2, "big"))
        s.sendall(hdr + mask + body)
        b0, payload = frame_payload(s, buf)
        assert b0 & 0x40, "reply is not compressed (RSV1 missing)"
        out = d.decompress(payload + b"\x00\x00\xff\xff")
        assert out == data, "reply differs after zlib inflate"
        sent.append(len(payload))
    assert sent[3] < sent[2], f"no context takeover: {sent}"
    print(f"zlib: 4 replies inflate as one stream, sizes {sent}")
    s.close()


if __name__ == "__main__":
    port = int(sys.argv[1])
    asyncio.run(interop(port))
    raw_zlib(port)
