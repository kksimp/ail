def build(lines):
    seen = set()
    for line in lines:
        p = line.split(":")
        seen.add(p[0].strip())
        for d in p[1].split():
            seen.add(d)
    names = sorted(seen)
    src = []
    dst = []
    for text in lines:
        q = text.split(":")
        task = names.index(q[0].strip())
        for d in q[1].split():
            src.append(names.index(d))
            dst.append(task)
    return names, src, dst


def order(g):
    names, src, dst = g
    n = len(names)
    indeg = [0] * n
    done = [False] * n
    for e in range(len(src)):
        indeg[dst[e]] += 1
    out = []
    while True:
        pick = -1
        for j in range(n):
            if not done[j] and indeg[j] == 0:
                pick = j
                break
        if pick == -1:
            break
        done[pick] = True
        out.append(pick)
        for e2 in range(len(src)):
            if src[e2] == pick:
                indeg[dst[e2]] -= 1
    return out


def report(title, lines):
    g = build(lines)
    out = order(g)
    names = [g[0][i] for i in out]
    print(f"{title}: {' '.join(names)}")
    if len(out) < len(g[0]):
        stuck = [g[0][k] for k in range(len(g[0])) if k not in out]
        print(f"  cycle among: {' '.join(stuck)}")


def main():
    report("build", [
        "app: lib ui",
        "ui: lib widgets",
        "lib: core",
        "widgets: core",
        "core:",
        "docs: app",
    ])
    report("cyclic", [
        "a: b",
        "b: c",
        "c: a",
        "d: a",
        "e:",
    ])


main()
