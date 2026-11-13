ALPHABET = "abcdefghijklmnopqrstuvwxyz"


class Trie:
    def __init__(self):
        self.edges = {}
        self.ends = [False]
        self.weights = [0]

    def insert(self, word, w):
        cur = 0
        for ch in word:
            key = f"{cur}:{ch}"
            if key in self.edges:
                cur = self.edges[key]
            else:
                nid = len(self.ends)
                self.ends.append(False)
                self.weights.append(0)
                self.edges[key] = nid
                cur = nid
        self.ends[cur] = True
        self.weights[cur] = w

    def find_node(self, prefix):
        cur = 0
        for ch in prefix:
            key = f"{cur}:{ch}"
            if key not in self.edges:
                return -1
            cur = self.edges[key]
        return cur

    def has(self, word):
        node = self.find_node(word)
        return node >= 0 and self.ends[node]

    def collect(self, node, text):
        out = []
        if self.ends[node]:
            out.append((text, self.weights[node]))
        for letter in ALPHABET:
            key = f"{node}:{letter}"
            if key in self.edges:
                out.extend(self.collect(self.edges[key], f"{text}{letter}"))
        return out

    def suggest(self, prefix, limit):
        node = self.find_node(prefix)
        if node < 0:
            return []
        hits = sorted(self.collect(node, prefix), key=lambda h: -h[1])
        return hits[:limit]


def show(hits):
    if not hits:
        return "(none)"
    return " ".join(f"{h[0]}({h[1]})" for h in hits)


def b(v):
    return "true" if v else "false"


def main():
    t = Trie()
    words = ["car", "card", "care", "careful", "cart", "cat", "catalog", "dog", "dot", "door", "do"]
    weights = [50, 20, 35, 5, 12, 40, 8, 30, 10, 15, 3]
    for i in range(len(words)):
        t.insert(words[i], weights[i])
    print(f"nodes: {len(t.ends)}")
    for prefix in ["car", "ca", "do", "d", "z", ""]:
        print(f"'{prefix}' -> {show(t.suggest(prefix, 3))}")
    for probe in ["car", "ca", "dot", "dots"]:
        print(f"has {probe}: {b(t.has(probe))}")


main()
