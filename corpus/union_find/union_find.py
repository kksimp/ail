class Dsu:
    def __init__(self, n):
        self.parent = list(range(n))
        self.sizes = [1] * n
        self.groups = n

    def find(self, x):
        cur = x
        while self.parent[cur] != cur:
            self.parent[cur] = self.parent[self.parent[cur]]
            cur = self.parent[cur]
        return cur

    def merge(self, a, b):
        ra = self.find(a)
        rb = self.find(b)
        if ra == rb:
            return False
        big = ra if self.sizes[ra] >= self.sizes[rb] else rb
        small = rb if big == ra else ra
        self.parent[small] = big
        self.sizes[big] = self.sizes[big] + self.sizes[small]
        self.groups -= 1
        return True


def kruskal(n, edges):
    sets = Dsu(n)
    tree = []
    for e in sorted(edges, key=lambda x: x[2]):
        if sets.merge(e[0], e[1]):
            tree.append(e)
    return tree


def b(v):
    return "true" if v else "false"


def main():
    labels = list("ABCDEFGHI")
    edges = [
        (0, 1, 7),
        (0, 3, 5),
        (1, 2, 8),
        (1, 3, 9),
        (1, 4, 7),
        (2, 4, 5),
        (3, 4, 15),
        (3, 5, 6),
        (4, 5, 8),
        (4, 6, 9),
        (5, 6, 11),
        (7, 8, 2),
    ]
    tree = kruskal(9, edges)
    total = 0
    for e in tree:
        total += e[2]
        print(f"{labels[e[0]]}-{labels[e[1]]} {e[2]}")
    print(f"tree edges: {len(tree)}")
    print(f"total weight: {total}")
    sets = Dsu(9)
    for e2 in tree:
        sets.merge(e2[0], e2[1])
    print(f"components: {sets.groups}")
    print(f"A and G connected: {b(sets.find(0) == sets.find(6))}")
    print(f"A and I connected: {b(sets.find(0) == sets.find(8))}")


main()
