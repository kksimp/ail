import os, tempfile


def print_tree(d, prefix, depth, t):
    subdirs = []
    plain = []
    for name in os.listdir(d):
        if os.path.isdir(os.path.join(d, name)):
            subdirs.append(name)
        else:
            plain.append(name)
    entries = sorted(subdirs) + sorted(plain)
    for i, name in enumerate(entries):
        last = i == len(entries) - 1
        branch = "`-- " if last else "|-- "
        is_dir = name in subdirs
        label = name + "/" if is_dir else name
        t["lines"].append(prefix + branch + label)
        if is_dir:
            t["dirs"] += 1
            if depth > 1:
                pad = "    " if last else "|   "
                print_tree(os.path.join(d, name), prefix + pad, depth - 1, t)
        else:
            t["files"] += 1


def render(root, depth):
    t = {"dirs": 0, "files": 0, "lines": []}
    print_tree(root, "", depth, t)
    return t


def build(root):
    os.makedirs(os.path.join(root, "src/net"))
    os.makedirs(os.path.join(root, "src/ui"))
    os.makedirs(os.path.join(root, "tests"))
    os.makedirs(os.path.join(root, "docs"))
    for rel in ["README", "src/main.ail", "src/net/http.ail", "src/net/tls.ail", "src/ui/window.ail", "tests/basic.ail", "docs/intro.md"]:
        with open(os.path.join(root, rel), "w", newline="") as f:
            f.write("x\n")


def remove_tree(d):
    for name in os.listdir(d):
        child = os.path.join(d, name)
        if os.path.isdir(child):
            remove_tree(child)
        else:
            os.remove(child)
    os.rmdir(d)


def main():
    root = tempfile.mkdtemp(prefix="corpus_tree_printer")
    build(root)
    full = render(root, 10)
    print(".")
    for l in full["lines"]:
        print(l)
    print(f"{full['dirs']} directories, {full['files']} files")
    shallow = render(root, 1)
    print("depth 1:")
    for l in shallow["lines"]:
        print(f"  {l}")
    print(f"{shallow['dirs']} directories, {shallow['files']} files")
    remove_tree(root)
    print(f"cleaned up: {str(not os.path.exists(root)).lower()}")


main()
