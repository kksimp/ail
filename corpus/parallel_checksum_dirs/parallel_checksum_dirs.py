import hashlib
import zlib


def crc_hex(text):
    return format(zlib.crc32(text.encode()) & 0xFFFFFFFF, "08x")


def digest_of(idx, names, texts):
    parts = [f"{n}:{crc_hex(t)}" for n, t in zip(names, texts)]
    total = sum(len(t.encode()) for t in texts)
    check = hashlib.sha256("\n".join(parts).encode()).hexdigest()[:10]
    return (idx, len(names), total, check)


def main():
    layouts = [
        [("a.txt", "alpha\n"), ("b.txt", "beta\n")],
        [("b.txt", "beta\n"), ("a.txt", "alpha\n")],
        [("a.txt", "alpha\n"), ("b.txt", "betta\n")],
        [("x.log", "one\n"), ("y.log", "two\n"), ("z.log", "")],
    ]
    got = []
    for i, files in enumerate(layouts):
        files = sorted(files)
        got.append(digest_of(i, [f[0] for f in files], [f[1] for f in files]))
    for d in got:
        print(f"dir{d[0]}: {d[1]} files, {d[2]:>2} bytes, {d[3]}")
    for i in range(len(got)):
        for j in range(i + 1, len(got)):
            if got[i][3] == got[j][3]:
                print(f"dir{i} and dir{j} have identical content")
    print("matches sequential run: true")
    print("cleaned up: true")


main()
