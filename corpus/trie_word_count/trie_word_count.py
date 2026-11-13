ALPHABET = "abcdefghijklmnopqrstuvwxyz"

TEXT = "the cat and the hat sat on the mat then the cat ran and the hat ran to the thatch"


class Trie:
    def __init__(self):
        self.edges = {}
        self.ends = [0]
        self.through = [0]

    def insert(self, word):
        cur = 0
        self.through[0] += 1
        for ch in word:
            key = f"{cur}:{ch}"
            if key in self.edges:
                cur = self.edges[key]
            else:
                nid = len(self.ends)
                self.ends.append(0)
                self.through.append(0)
                self.edges[key] = nid
                cur = nid
            self.through[cur] += 1
        self.ends[cur] += 1

    def node_of(self, prefix):
        cur = 0
        for ch in prefix:
            key = f"{cur}:{ch}"
            if key not in self.edges:
                return -1
            cur = self.edges[key]
        return cur

    def count_word(self, word):
        node = self.node_of(word)
        if node < 0:
            return 0
        return self.ends[node]

    def count_prefix(self, prefix):
        node = self.node_of(prefix)
        if node < 0:
            return 0
        return self.through[node]

    def walk(self, node, text):
        out = []
        if self.ends[node] > 0:
            out.append(f"{text}={self.ends[node]}")
        for letter in ALPHABET:
            key = f"{node}:{letter}"
            if key in self.edges:
                out.extend(self.walk(self.edges[key], text + letter))
        return out


t = Trie()
for w in TEXT.split(" "):
    t.insert(w)
print(f"words: {t.count_prefix('')}, nodes: {len(t.ends)}")
print(" ".join(t.walk(0, "")))
for p in ["th", "t", "ca", "h", "x", "the", "that"]:
    print(f"prefix {p}: {t.count_prefix(p)} words, exact {t.count_word(p)}")
