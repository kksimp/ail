INF = 1000000000


def bellman_ford(n, edges, src):
    dist = [INF] * n
    prev = [-1] * n
    dist[src] = 0
    rounds = 0
    for rnd in range(1, n):
        changed = False
        for a, b, w in edges:
            if dist[a] != INF and dist[a] + w < dist[b]:
                dist[b] = dist[a] + w
                prev[b] = a
                changed = True
        rounds = rnd
        if not changed:
            break
    cycle = False
    for a, b, w in edges:
        if dist[a] != INF and dist[a] + w < dist[b]:
            cycle = True
    return dist, prev, rounds, cycle


def path_to(prev, names, target):
    hops = [names[target]]
    cur = target
    while prev[cur] != -1:
        cur = prev[cur]
        hops.append(names[cur])
    hops.reverse()
    return " -> ".join(hops)


def parse_edges(names, lines):
    out = []
    for line in lines:
        p = line.split(" ")
        out.append((names.index(p[0]), names.index(p[1]), int(p[2])))
    return out


def report(title, names, lines):
    edges = parse_edges(names, lines)
    dist, prev, rounds, cycle = bellman_ford(len(names), edges, 0)
    print(f"{title}: {len(edges)} edges, {rounds} rounds")
    if cycle:
        print(f"  negative cycle reachable from {names[0]}")
        return
    for i in range(len(names)):
        if dist[i] == INF:
            print(f"  {names[i]}: unreachable")
        else:
            print(f"  {names[i]}: {dist[i]} via {path_to(prev, names, i)}")


def main():
    names = ["S", "A", "B", "C", "D", "E", "F"]
    report("currency-like", names, [
        "S A 6",
        "S B 7",
        "A B 8",
        "A C 5",
        "A D -4",
        "B C -3",
        "B D 9",
        "C A -2",
        "D S 2",
        "D C 7",
        "E F 1",
    ])
    report("with a cycle", names, [
        "S A 1",
        "A B -2",
        "B C -2",
        "C A 1",
        "C D 4",
    ])


main()
