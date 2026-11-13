import os, shutil, tempfile, zlib


def digest(data):
    return (len(data), format(zlib.crc32(data) & 0xFFFFFFFF, "x").rjust(8, "0"))


def manifest(d):
    m = {}
    for name in os.listdir(d):
        with open(os.path.join(d, name), "rb") as f:
            m[name] = digest(f.read())
    return m


def diff(old, new):
    lines = []
    for name in sorted(set(old) | set(new)):
        if name not in old:
            lines.append(f"+ {name} ({new[name][0]} bytes)")
        elif name not in new:
            lines.append(f"- {name}")
        elif old[name] != new[name]:
            lines.append(f"~ {name} {old[name][1]} -> {new[name][1]}")
        else:
            lines.append(f"= {name}")
    return lines


def write_all(d, names, texts):
    for n, t in zip(names, texts):
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(t)


def main():
    root = tempfile.mkdtemp(prefix="corpus_file_manifest_diff")
    a = os.path.join(root, "v1")
    b = os.path.join(root, "v2")
    os.mkdir(a)
    os.mkdir(b)
    write_all(a, ["main.c", "util.c", "README", "old.txt"], ["int main() {}\n", "void u() {}\n", "docs\n", "legacy\n"])
    write_all(b, ["main.c", "util.c", "README", "new.txt"], ["int main() { return 0; }\n", "void u() {}\n", "docs\n", "fresh data\n"])
    ma = manifest(a)
    mb = manifest(b)
    print(f"v1: {len(ma)} files, v2: {len(mb)} files")
    lines = diff(ma, mb)
    for l in lines:
        print(l)
    changes = [l for l in lines if not l.startswith("=")]
    print(f"{len(changes)} of {len(lines)} entries differ")
    shutil.rmtree(root)
    print(f"cleaned up: {str(not os.path.exists(root)).lower()}")


main()
