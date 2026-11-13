import hashlib, os, tempfile, zlib


def chunk_digests(p, chunk_size):
    with open(p, "rb") as f:
        data = f.read()
    out = []
    off = 0
    idx = 0
    while off < len(data):
        n = min(chunk_size, len(data) - off)
        crc = format(zlib.crc32(data[off:off + n]) & 0xFFFFFFFF, "x").rjust(8, "0")
        out.append((idx, off, n, crc))
        off += n
        idx += 1
    return out


def changed_chunks(before, after):
    out = []
    for i in range(max(len(before), len(after))):
        if i >= len(before) or i >= len(after) or before[i][3] != after[i][3]:
            out.append(i)
    return out


def root_digest(chunks):
    return hashlib.sha256(",".join(c[3] for c in chunks).encode()).hexdigest()[:16]


def show(chunks):
    for idx, off, size, crc in chunks:
        print(f"  chunk {idx} offset {off:>4} size {size:>3} crc {crc}")
    print(f"  root {root_digest(chunks)}")


def main():
    d = tempfile.mkdtemp(prefix="corpus_chunked_checksum")
    p = os.path.join(d, "records.dat")
    with open(p, "w", newline="") as f:
        for i in range(1, 101):
            f.write(f"rec {i:04} {i * i % 997:03}\n")
    print(f"size {os.path.getsize(p)}")
    first = chunk_digests(p, 500)
    show(first)
    with open(p, "r+b") as f:
        f.seek(620)
        f.write(b"#")
    second = chunk_digests(p, 500)
    print(f"after patching byte 620: changed {changed_chunks(first, second)}")
    with open(p, "a", newline="") as f:
        f.write("end of file\n")
    third = chunk_digests(p, 500)
    print(f"after appending: changed {changed_chunks(second, third)}")
    show(third)
    os.remove(p)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
