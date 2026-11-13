INF = 1000000000


def build(lines):
    seen = set()
    for line in lines:
        p = line.split(" ")
        seen.add(p[0])
        seen.add(p[1])
    names = sorted(seen)
    g = {"names": names, "from": [], "to": [], "cost": []}
    for line in lines:
        p = line.split(" ")
        a = names.index(p[0])
        b = names.index(p[1])
        c = int(p[2])
        g["from"] += [a, b]
        g["to"] += [b, a]
        g["cost"] += [c, c]
    return g


def shortest(g, src):
    count = len(g["names"])
    dist = [INF] * count
    prev = [-1] * count
    done = [False] * count
    dist[src] = 0
    for _ in range(count):
        u = -1
        for i in range(count):
            if not done[i] and dist[i] < INF and (u == -1 or dist[i] < dist[u]):
                u = i
        if u == -1:
            break
        done[u] = True
        for e in range(len(g["from"])):
            t = g["to"][e]
            if g["from"][e] == u and dist[u] + g["cost"][e] < dist[t]:
                dist[t] = dist[u] + g["cost"][e]
                prev[t] = u
    return dist, prev


def route(g, prev, target):
    hops = [g["names"][target]]
    cur = target
    while prev[cur] != -1:
        cur = prev[cur]
        hops.append(g["names"][cur])
    hops.reverse()
    return " -> ".join(hops)


def main():
    lines = [
        "A B 4", "A C 2", "B C 1", "B D 5", "C D 8", "C E 10",
        "D E 2", "D F 6", "E F 3", "X Y 2",
    ]
    g = build(lines)
    src = g["names"].index("A")
    dist, prev = shortest(g, src)
    for i in range(len(g["names"])):
        if dist[i] == INF:
            print(f"{g['names'][i]}: unreachable")
        else:
            print(f"{g['names'][i]}: {dist[i]} via {route(g, prev, i)}")


main()
