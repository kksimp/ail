INF = 1000000000


def build(n, edges):
    dist = [INF] * (n * n)
    nxt = [-1] * (n * n)
    for v in range(n):
        dist[v * n + v] = 0
        nxt[v * n + v] = v
    for e in edges:
        dist[e[0] * n + e[1]] = e[2]
        nxt[e[0] * n + e[1]] = e[1]
    return (n, dist, nxt)


def run(net):
    n, dist, nxt = net
    dist = list(dist)
    nxt = list(nxt)
    for k in range(n):
        for i in range(n):
            if dist[i * n + k] == INF:
                continue
            for j in range(n):
                if dist[k * n + j] == INF:
                    continue
                through = dist[i * n + k] + dist[k * n + j]
                if through < dist[i * n + j]:
                    dist[i * n + j] = through
                    nxt[i * n + j] = nxt[i * n + k]
    return (n, dist, nxt)


def negative_nodes(net):
    n, dist, nxt = net
    return [v for v in range(n) if dist[v * n + v] < 0]


def route(net, names, from_node, to_node):
    n, dist, nxt = net
    if dist[from_node * n + to_node] == INF:
        return "no route"
    cur = from_node
    hops = [names[cur]]
    while cur != to_node:
        cur = nxt[cur * n + to_node]
        hops.append(names[cur])
    return " -> ".join(hops)


def print_matrix(net, names):
    n, dist, nxt = net
    header = [s.rjust(5) for s in names]
    print("     " + "".join(header))
    for i in range(n):
        cells = []
        for j in range(n):
            d = dist[i * n + j]
            text = "inf" if d == INF else str(d)
            cells.append(text.rjust(5))
        print(names[i].ljust(5) + "".join(cells))


def main():
    names = ["A", "B", "C", "D", "E"]
    edges = [
        [0, 1, 3],
        [0, 2, 8],
        [0, 4, -4],
        [1, 3, 1],
        [1, 4, 7],
        [2, 1, 4],
        [3, 0, 2],
        [3, 2, -5],
        [4, 3, 6],
    ]
    solved = run(build(5, edges))
    print_matrix(solved, names)
    for pair in [[0, 3], [3, 1], [4, 2], [0, 4]]:
        print(f"{names[pair[0]]} to {names[pair[1]]}: {solved[1][pair[0] * 5 + pair[1]]} via {route(solved, names, pair[0], pair[1])}")
    broken = run(build(4, [[0, 1, 1], [1, 2, -3], [2, 1, 1], [2, 3, 2]]))
    bad = negative_nodes(broken)
    print("negative cycle through nodes: " + " ".join(str(v) for v in bad))
    one_way = run(build(3, [[0, 1, 5], [1, 2, 5]]))
    print(f"2 to 0: {route(one_way, ['x', 'y', 'z'], 2, 0)}")


main()
