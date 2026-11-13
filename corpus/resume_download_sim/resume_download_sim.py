import hashlib, os, tempfile, zlib


def make_source(blocks):
    parts = []
    for i in range(blocks):
        parts.append(f"block {i:03}: {'abcdefghij'[i % 10:10]}-{i * 7919 % 1000:03}\n")
    return "".join(parts)


def fetch(source, off, count):
    return source[off:min(off + count, len(source))]


def download(source, p, chunk, budget):
    if not os.path.exists(p):
        open(p, "w").close()
    start = os.path.getsize(p)
    off = start
    used = 0
    with open(p, "ab") as w:
        while off < len(source) and used < budget:
            piece = fetch(source, off, chunk)
            w.write(piece.encode())
            off += len(piece.encode())
            used += 1
    return (start, off - start, off >= len(source))


def main():
    d = tempfile.mkdtemp(prefix="corpus_resume_download_sim")
    p = os.path.join(d, "payload.part")
    source = make_source(60)
    print(f"source: {len(source)} bytes, crc {format(zlib.crc32(source.encode()), 'x')}")
    budgets = [4, 3, 5, 100]
    sent = 0
    restart_cost = 0
    for i in range(len(budgets)):
        start, got, done = download(source, p, 128, budgets[i])
        sent += got
        restart_cost += start + got
        print(f"attempt {i + 1}: resumed at {start:>4}, received {got:>4}, done {str(done).lower()}")
        if done:
            break
    print(f"bytes on the wire: {sent}")
    print(f"bytes if every retry restarted: {restart_cost}")
    with open(p, "rb") as f:
        ok = f.read() == source.encode()
    print(f"verified: {str(ok).lower()}")
    os.remove(p)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
