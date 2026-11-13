import os, tempfile, shutil


def extension(name):
    parts = name.split(".")
    if len(parts) == 1:
        return "(none)"
    return parts[-1]


def measure(d, depth, by_ext, files, lines):
    total = 0
    for name in sorted(os.listdir(d)):
        child = os.path.join(d, name)
        if os.path.isdir(child):
            slot = len(lines)
            lines.append("")
            sub = measure(child, depth + 1, by_ext, files, lines)
            lines[slot] = f"{'  ' * depth}{name}/ {sub}"
            total += sub
        else:
            size = os.path.getsize(child)
            e = extension(name)
            by_ext[e] = by_ext.get(e, 0) + size
            files.append((name, size))
            total += size
    return total


def add_file(root, rel, size):
    with open(os.path.join(root, rel), "w", newline="") as f:
        f.write("x" * size)


def build(root):
    os.makedirs(os.path.join(root, "src/util"))
    os.makedirs(os.path.join(root, "assets"))
    add_file(root, "README", 120)
    add_file(root, "src/main.ail", 900)
    add_file(root, "src/lexer.ail", 1500)
    add_file(root, "src/util/strings.ail", 640)
    add_file(root, "src/util/notes.txt", 75)
    add_file(root, "assets/logo.png", 2048)
    add_file(root, "assets/icon.png", 512)


def main():
    root = tempfile.mkdtemp(prefix="corpus_dir_size_summary")
    build(root)
    by_ext = {}
    files = []
    lines = []
    total = measure(root, 0, by_ext, files, lines)
    for l in lines:
        print(l)
    print(f"total {total} bytes in {len(files)} files")
    for ext in sorted(by_ext):
        print(f"{ext:<8}{by_ext[ext]:>6}")
    big = sorted(files, key=lambda f: -f[1])[0]
    print(f"largest: {big[0]} ({big[1]})")
    shutil.rmtree(root)
    print(f"cleaned up: {str(not os.path.exists(root)).lower()}")


main()
