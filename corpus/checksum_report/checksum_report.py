import hashlib, os, tempfile, zlib


def digest(data):
    crc = format(zlib.crc32(data) & 0xFFFFFFFF, "x").rjust(8, "0")
    return (len(data), crc, hashlib.sha256(data).hexdigest()[:12])


def manifest(d):
    m = {}
    for name in os.listdir(d):
        with open(os.path.join(d, name), "rb") as f:
            m[name] = digest(f.read())
    return m


def verify(d, expected):
    changed = []
    for name in sorted(expected):
        p = os.path.join(d, name)
        if not os.path.exists(p):
            changed.append(f"{name} (missing)")
        else:
            with open(p, "rb") as f:
                if digest(f.read()) != expected[name]:
                    changed.append(name)
    return changed


def fmt_list(xs):
    return ", ".join(xs) if xs else "none"


def put(p, text):
    with open(p, "w", newline="") as f:
        f.write(text)


def main():
    d = tempfile.mkdtemp(prefix="corpus_checksum_report")
    names = ["notes.txt", "data.csv", "empty.txt", "config.ini"]
    texts = ["remember the milk\n", "a,b\n1,2\n3,4\n", "", "mode=fast\nretries=3\n"]
    for n, t in zip(names, texts):
        put(os.path.join(d, n), t)
    m = manifest(d)
    print(f"{'file':<11}{'size':>5}  {'crc32':<9}{'sha256':<12}")
    for name in sorted(m):
        size, crc, sha = m[name]
        print(f"{name:<11}{size:>5}  {crc:<9}{sha:<12}")
    print(f"verify (untouched): {fmt_list(verify(d, m))}")
    put(os.path.join(d, "data.csv"), "a,b\n1,2\n3,5\n")
    os.remove(os.path.join(d, "empty.txt"))
    print(f"verify (after edits): {fmt_list(verify(d, m))}")
    for name in os.listdir(d):
        os.remove(os.path.join(d, name))
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
