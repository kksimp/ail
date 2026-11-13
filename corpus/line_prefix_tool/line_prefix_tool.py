import os, tempfile


def apply(e, line, idx):
    kind, arg = e
    if kind == "numbered":
        return f"{str(idx).rjust(arg)}  {line}"
    if kind == "prefix":
        return f"{arg}{line}"
    if kind == "strip":
        return line[len(arg):] if line.startswith(arg) else line
    if kind == "indent":
        return line if line == "" else " " * arg + line
    return "#" if line == "" else f"# {line}"


def apply_all(edits, line, idx):
    cur = line
    for e in edits:
        cur = apply(e, cur, idx)
    return cur


def transform(src, dst, edits):
    n = 0
    with open(src, newline="") as r, open(dst, "w", newline="") as w:
        for line in r:
            line = line.rstrip("\n")
            n += 1
            w.write(apply_all(edits, line, n) + "\n")
    return n


def show(title, p):
    print(title)
    with open(p, newline="") as f:
        for line in f.read().splitlines():
            print(f"|{line}")


def main():
    d = tempfile.mkdtemp(prefix="corpus_line_prefix_tool")
    src = os.path.join(d, "in.txt")
    with open(src, "w", newline="") as f:
        f.write("alpha\n> beta\n\n> gamma\ndelta\n")
    out = os.path.join(d, "out.txt")
    n = transform(src, out, [("strip", "> "), ("indent", 2), ("numbered", 3)])
    print(f"pass 1: {n} lines")
    show("numbered and indented:", out)
    n2 = transform(src, out, [("comment", None), ("prefix", "//")])
    print(f"pass 2: {n2} lines")
    show("commented:", out)
    os.remove(src)
    os.remove(out)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
