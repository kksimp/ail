INF = 1000000


class Tree:
    def __init__(self):
        self.edges = []
        self.total = 0
        self.covered = 0


def matrix(n, text):
    w = [INF for _ in range(n * n)]
    for tok in text.split():
        p = tok.split(":")
        ends = p[0].split("-")
        a = int(ends[0])
        b = int(ends[1])
        cost = int(p[1])
        w[a * n + b] = cost
        w[b * n + a] = cost
    return w


def prim(n, w):
    best = [INF for _ in range(n)]
    from_v = [-1 for _ in range(n)]
    inside = [False for _ in range(n)]
    tree = Tree()
    best[0] = 0
    for _round in range(n):
        pick = -1
        for v in range(n):
            if not inside[v] and best[v] < INF:
                if pick == -1 or best[v] < best[pick]:
                    pick = v
        if pick == -1:
            break
        inside[pick] = True
        tree.covered += 1
        if from_v[pick] != -1:
            tree.edges.append(f"{from_v[pick]}-{pick} ({best[pick]})")
            tree.total += best[pick]
        for v2 in range(n):
            cost = w[pick * n + v2]
            if not inside[v2] and cost < best[v2]:
                best[v2] = cost
                from_v[v2] = pick
    return tree


def report(title, n, text):
    t = prim(n, matrix(n, text))
    print(f"{title}: {t.covered} of {n} vertices, weight {t.total}")
    for e in t.edges:
        print(f"  {e}")


def main():
    report("campus", 6, "0-1:4 0-2:3 1-2:1 1-3:2 2-3:4 3-4:2 4-5:6 3-5:7")
    report("ring", 5, "0-1:10 1-2:10 2-3:10 3-4:10 4-0:5")
    report("split", 5, "0-1:2 1-2:3 3-4:1")


main()
