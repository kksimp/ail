import os, shutil, tempfile


def extension(name):
    parts = name.split(".")
    if len(parts) == 1:
        return "none"
    return parts[-1]


def walk(d, rel, depth, st):
    names = sorted(os.listdir(d))
    if not names:
        st["empty"] += 1
    if len(names) > st["busy_n"]:
        st["busy"] = "." if rel == "" else rel
        st["busy_n"] = len(names)
    for name in names:
        child = os.path.join(d, name)
        label = name if rel == "" else f"{rel}/{name}"
        while len(st["files"]) <= depth:
            st["files"].append(0)
            st["dirs"].append(0)
        if os.path.isdir(child):
            st["dirs"][depth] += 1
            walk(child, label, depth + 1, st)
        else:
            st["files"][depth] += 1
            e = extension(name)
            st["ext"][e] = st["ext"].get(e, 0) + 1
            if depth > st["deep_d"]:
                st["deep"] = label
                st["deep_d"] = depth


def main():
    root = tempfile.mkdtemp(prefix="corpus_directory_tree_stats")
    for d in ["src/core/deep", "docs", "empty/inner", "tests"]:
        os.makedirs(os.path.join(root, d))
    for rel in ["Makefile", "src/main.ail", "src/util.ail", "src/core/lex.ail", "src/core/deep/x.tmp", "docs/a.md", "docs/b.md", "docs/c.md", "tests/t1.ail"]:
        with open(os.path.join(root, rel), "w", newline="") as f:
            f.write("data")
    st = {"files": [], "dirs": [], "empty": 0, "deep": "", "deep_d": -1, "ext": {}, "busy": "", "busy_n": 0}
    walk(root, "", 0, st)
    for d in range(len(st["files"])):
        dirs = st["dirs"][d] if d < len(st["dirs"]) else 0
        print(f"depth {d}: {st['files'][d]} files, {dirs} dirs")
    print(f"empty dirs: {st['empty']}")
    print(f"deepest file: {st['deep']} (depth {st['deep_d']})")
    print(f"busiest dir: {st['busy']} with {st['busy_n']} entries")
    for e in sorted(st["ext"]):
        print(f"  .{e}: {st['ext'][e]}")
    shutil.rmtree(root)
    print(f"cleaned up: {str(not os.path.exists(root)).lower()}")


main()
