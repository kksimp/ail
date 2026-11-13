class Digraph:
    def __init__(self, n, src, dst):
        self.n = n
        self.src = src
        self.dst = dst


def parse(n, text):
    g = Digraph(n, [], [])
    for tok in text.split():
        p = tok.split(">")
        g.src.append(int(p[0]))
        g.dst.append(int(p[1]))
    return g


def visit(g, v, color, path):
    color[v] = 1
    path.append(v)
    for e in range(len(g.src)):
        if g.src[e] != v:
            continue
        w = g.dst[e]
        if color[w] == 1:
            path.append(w)
            return True
        if color[w] == 0 and visit(g, w, color, path):
            return True
    color[v] = 2
    path.pop()
    return False


def find_cycle(g):
    color = [0 for _ in range(g.n)]
    path = []
    for v in range(g.n):
        if color[v] == 0 and visit(g, v, color, path):
            first = path.index(path[len(path) - 1])
            return path[first:]
    return []


def describe(title, n, text):
    cyc = find_cycle(parse(n, text))
    if not cyc:
        print(f"{title}: acyclic")
    else:
        print(f"{title}: cycle of length {len(cyc) - 1}: {' -> '.join(str(x) for x in cyc)}")


def main():
    describe("build order", 6, "0>1 0>2 1>3 2>3 3>4 4>5")
    describe("deadlock", 5, "0>1 1>2 2>3 3>1 3>4")
    describe("self loop", 3, "0>1 1>1")
    describe("long ring", 6, "0>1 1>2 2>3 3>4 4>5 5>0")
    describe("diamond plus back edge", 5, "0>1 0>2 1>3 2>3 3>4 4>2")
    describe("empty", 3, "")


main()
