class Bst:
    def __init__(self):
        self.keys = []
        self.left = []
        self.right = []

    def insert(self, k):
        nid = len(self.keys)
        if nid == 0:
            self.keys.append(k)
            self.left.append(-1)
            self.right.append(-1)
            return True
        cur = 0
        while True:
            if k == self.keys[cur]:
                return False
            if k < self.keys[cur]:
                if self.left[cur] == -1:
                    self.left[cur] = nid
                    break
                cur = self.left[cur]
            else:
                if self.right[cur] == -1:
                    self.right[cur] = nid
                    break
                cur = self.right[cur]
        self.keys.append(k)
        self.left.append(-1)
        self.right.append(-1)
        return True

    def depth_of(self, k):
        if len(self.keys) == 0:
            return -1
        cur = 0
        d = 0
        while cur != -1:
            if k == self.keys[cur]:
                return d
            cur = self.left[cur] if k < self.keys[cur] else self.right[cur]
            d += 1
        return -1

    def inorder(self, i):
        if i == -1:
            return []
        out = self.inorder(self.left[i])
        out.append(self.keys[i])
        out.extend(self.inorder(self.right[i]))
        return out

    def height(self, i):
        if i == -1:
            return 0
        lh = self.height(self.left[i])
        rh = self.height(self.right[i])
        return 1 + max(lh, rh)


def next_rand(s):
    return (s * 1103515245 + 12345) % 2147483648


def main():
    tree = Bst()
    seed = 7
    dups = 0
    for i in range(20):
        seed = next_rand(seed)
        if not tree.insert(seed // 65536 % 50):
            dups += 1
    sorted_keys = tree.inorder(0)
    print(f"keys in order: {sorted_keys}")
    print(f"nodes: {len(tree.keys)}, duplicates skipped: {dups}")
    print(f"height: {tree.height(0)}")
    print(f"min: {sorted_keys[0]}, max: {sorted_keys[-1]}")
    for probe in [tree.keys[0], 25, 49, 100]:
        print(f"depth of {probe}: {tree.depth_of(probe)}")
    below = [k for k in sorted_keys if k < 25]
    print(f"keys below 25: {len(below)}")


main()
