import hashlib, os, shutil, tempfile


def dupe_groups(names, texts):
    by_hash = {}
    for n, t in zip(names, texts):
        by_hash.setdefault(hashlib.sha256(t.encode()).hexdigest(), []).append(n)
    groups = [g for g in by_hash.values() if len(g) > 1]
    return sorted(groups)


def main():
    d = tempfile.mkdtemp(prefix="corpus_file_dedupe")
    names = ["a.txt", "b.txt", "c.log", "d.txt", "e.cfg", "f.log", "g.txt"]
    texts = [
        "alpha beta gamma\n",
        "unique content here\n",
        "retry=3\ntimeout=30\n",
        "alpha beta gamma\n",
        "retry=3\ntimeout=30\n",
        "retry=3\ntimeout=30\n",
        "alpha beta gamma!\n",
    ]
    for n, t in zip(names, texts):
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(t)
    files = sorted(os.path.join(d, n) for n in os.listdir(d))
    found = []
    contents = []
    for p in files:
        found.append(os.path.basename(p))
        with open(p, newline="") as f:
            contents.append(f.read())
    print(f"scanned {len(found)} files")
    groups = dupe_groups(found, contents)
    wasted = 0
    for i, g in enumerate(groups):
        size = len(contents[found.index(g[0])].encode())
        wasted += size * (len(g) - 1)
        print(f"group {i + 1}: {', '.join(g)} ({size} bytes each)")
    print(f"{len(groups)} duplicate groups, {wasted} bytes wasted")
    shutil.rmtree(d)


main()
