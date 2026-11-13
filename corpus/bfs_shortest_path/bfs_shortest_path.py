class Net:
    def __init__(self, names, ea, eb):
        self.names = names
        self.ea = ea
        self.eb = eb


def build(letters, links):
    names = [c for c in letters]
    net = Net(names, [], [])
    for link in links:
        parts = link.split("-")
        net.ea.append(names.index(parts[0]))
        net.eb.append(names.index(parts[1]))
    return net


def neighbors(net, v):
    out = []
    for e in range(len(net.ea)):
        if net.ea[e] == v:
            out.append(net.eb[e])
        elif net.eb[e] == v:
            out.append(net.ea[e])
    return out


def search(net, start):
    prev = [-2 for _ in range(len(net.names))]
    prev[start] = -1
    queue = [start]
    head = 0
    while head < len(queue):
        cur = queue[head]
        head += 1
        for nb in neighbors(net, cur):
            if prev[nb] == -2:
                prev[nb] = cur
                queue.append(nb)
    return prev


def route(prev, goal):
    path = []
    if prev[goal] == -2:
        return path
    here = goal
    while here != -1:
        path.append(here)
        here = prev[here]
    path.reverse()
    return path


def show_route(net, src, dst):
    a = net.names.index(src)
    b = net.names.index(dst)
    path = route(search(net, a), b)
    if not path:
        print(f"{src} to {dst}: unreachable")
        return
    stops = [net.names[i] for i in path]
    print(f"{src} to {dst}: {len(path) - 1} hops via {' > '.join(stops)}")


def main():
    net = build("ABCDEFGHIJ", [
        "A-B", "A-C", "B-D", "C-D", "D-E", "E-F",
        "C-G", "G-F", "F-H", "I-J",
    ])
    prev = search(net, 0)
    for i in range(len(net.names)):
        if prev[i] == -2:
            print(f"{net.names[i]}: not reachable from A")
        else:
            print(f"{net.names[i]}: {len(route(prev, i)) - 1} hops from A")
    show_route(net, "A", "H")
    show_route(net, "B", "G")
    show_route(net, "H", "A")
    show_route(net, "A", "J")
    show_route(net, "I", "J")


main()
