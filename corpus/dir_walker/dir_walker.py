import os, shutil, tempfile


def visit(d, depth, st):
    for name in sorted(os.listdir(d)):
        child = os.path.join(d, name)
        pad = "  " * depth
        if os.path.isdir(child):
            st["dirs"] += 1
            st["lines"].append(f"{pad}{name}/")
            visit(child, depth + 1, st)
        else:
            size = os.path.getsize(child)
            st["files"] += 1
            st["bytes"] += size
            st["lines"].append(f"{pad}{name} ({size})")


def build(root):
    os.makedirs(os.path.join(root, "src/core"))
    os.makedirs(os.path.join(root, "docs"))
    os.makedirs(os.path.join(root, "empty"))
    files = {
        "README.md": "# demo\n",
        "src/main.txt": "entry point\n",
        "src/core/engine.txt": "engine code goes here\n",
        "src/core/util.txt": "helpers\n",
        "docs/guide.txt": "read me first\nthen this\n",
    }
    for name, text in files.items():
        with open(os.path.join(root, name), "w", newline="") as f:
            f.write(text)


def main():
    root = tempfile.mkdtemp(prefix="corpus_dir_walker")
    build(root)
    st = {"files": 0, "dirs": 0, "bytes": 0, "lines": []}
    visit(root, 0, st)
    for line in st["lines"]:
        print(line)
    print(f"{st['files']} files, {st['dirs']} dirs, {st['bytes']} bytes")
    n = sum(len(fs) for _, _, fs in os.walk(root))
    print(f"walk finds {n} files")
    shutil.rmtree(root)
    print(f"cleaned up: {str(not os.path.exists(root)).lower()}")


main()
