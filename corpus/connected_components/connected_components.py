class Graph:
    def __init__(self, n, ea, eb):
        self.n = n
        self.ea = ea
        self.eb = eb


def parse(n, text):
    g = Graph(n, [], [])
    for tok in text.split():
        p = tok.split("-")
        g.ea.append(int(p[0]))
        g.eb.append(int(p[1]))
    return g


def components(g):
    label = [-1 for _ in range(g.n)]
    groups = []
    for start in range(g.n):
        if label[start] != -1:
            continue
        gid = len(groups)
        members = []
        stack = [start]
        label[start] = gid
        while len(stack) > 0:
            v = stack.pop()
            members.append(v)
            for e in range(len(g.ea)):
                other = -1
                if g.ea[e] == v:
                    other = g.eb[e]
                elif g.eb[e] == v:
                    other = g.ea[e]
                if other >= 0 and label[other] == -1:
                    label[other] = gid
                    stack.append(other)
        members.sort()
        groups.append(members)
    return groups


def main():
    g = parse(12, "0-1 1-2 2-0 3-4 5-6 6-7 7-8 8-5 5-7 10-11")
    groups = components(g)
    print(f"vertices: {g.n}, edges: {len(g.ea)}")
    print(f"components: {len(groups)}")
    for i in range(len(groups)):
        print(f"  #{i + 1} size {len(groups[i])}: {' '.join(str(x) for x in groups[i])}")
    ranked = sorted(groups, key=lambda m: -len(m))
    print(f"largest has {len(ranked[0])} vertices")
    singles = [m[0] for m in groups if len(m) == 1]
    print(f"isolated: {' '.join(str(x) for x in singles)}")
    joined = parse(12, "0-1 1-2 2-0 3-4 5-6 6-7 7-8 8-5 5-7 10-11 2-3 4-5 8-9 9-10 11-0")
    print(f"after bridging: {len(components(joined))} component")


main()
