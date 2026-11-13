import os, tempfile


def split_name(name):
    parts = name.split(".")
    if len(parts) < 2:
        return (name, "")
    return (".".join(parts[:-1]), parts[-1])


def join_name(stem, ext):
    if ext == "":
        return stem
    return f"{stem}.{ext}"


def clean(stem):
    out = []
    for c in stem.lower():
        if c in " -_":
            out.append("_")
        elif c.isalpha() or c.isdigit():
            out.append(c)
    return "".join(out)


def plan(names):
    taken = set()
    moves = []
    for name in names:
        stem, ext = split_name(name)
        base = clean(stem)
        n = 1
        cand = join_name(base, ext.lower())
        while cand in taken:
            n += 1
            cand = join_name(f"{base}_{n}", ext.lower())
        taken.add(cand)
        moves.append((name, cand))
    return moves


def main():
    names = [
        "Report Final.TXT",
        "report-final.txt",
        "notes (draft).md",
        "Notes-Draft.md",
        "IMG 001.png",
        "img_002.png",
        "already_ok.txt",
        "Build Notes",
    ]
    moves = plan(names)
    for src, dst in moves:
        mark = "=" if src == dst else ">"
        print(f"{mark} {src:<18}-> {dst}")
    d = tempfile.mkdtemp(prefix="corpus_file_rename_plan")
    for n in names:
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(n + "\n")
    renamed = 0
    for src, dst in moves:
        if src != dst:
            os.rename(os.path.join(d, src), os.path.join(d, dst))
            renamed += 1
    print(f"renamed {renamed} of {len(moves)} files")
    for n in sorted(os.listdir(d)):
        print(f"  {n}")
        os.remove(os.path.join(d, n))
    os.rmdir(d)


main()
