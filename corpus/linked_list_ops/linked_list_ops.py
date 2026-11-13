class LinkedList:
    def __init__(self):
        self.vals = []
        self.nexts = []
        self.head = -1
        self.size = 0

    def push_front(self, x):
        self.vals.append(x)
        self.nexts.append(self.head)
        self.head = len(self.vals) - 1
        self.size += 1

    def node_at(self, i):
        cur = self.head
        for _ in range(i):
            cur = self.nexts[cur]
        return cur

    def push_back(self, x):
        if self.size == 0:
            self.push_front(x)
            return
        last = self.node_at(self.size - 1)
        self.vals.append(x)
        self.nexts.append(-1)
        self.nexts[last] = len(self.vals) - 1
        self.size += 1

    def insert_at(self, i, x):
        if i == 0:
            self.push_front(x)
            return
        prev = self.node_at(i - 1)
        self.vals.append(x)
        self.nexts.append(self.nexts[prev])
        self.nexts[prev] = len(self.vals) - 1
        self.size += 1

    def remove_at(self, i):
        if i == 0:
            self.head = self.nexts[self.head]
        else:
            prev = self.node_at(i - 1)
            self.nexts[prev] = self.nexts[self.nexts[prev]]
        self.size -= 1

    def reverse(self):
        prev = -1
        cur = self.head
        while cur != -1:
            nx = self.nexts[cur]
            self.nexts[cur] = prev
            prev = cur
            cur = nx
        self.head = prev

    def to_list(self):
        out = []
        cur = self.head
        while cur != -1:
            out.append(self.vals[cur])
            cur = self.nexts[cur]
        return out

    def middle(self):
        slow = self.head
        fast = self.head
        while fast != -1 and self.nexts[fast] != -1:
            slow = self.nexts[slow]
            fast = self.nexts[self.nexts[fast]]
        return slow


def int_list(xs):
    l = LinkedList()
    for x in xs:
        l.push_back(x)
    return l


def merge_sorted(a, b):
    xs = a.to_list()
    ys = b.to_list()
    out = LinkedList()
    i = 0
    j = 0
    while i < len(xs) or j < len(ys):
        take_left = j >= len(ys) or (i < len(xs) and xs[i] <= ys[j])
        if take_left:
            out.push_back(xs[i])
            i += 1
        else:
            out.push_back(ys[j])
            j += 1
    return out


nums = int_list([4, 8, 15, 16, 23])
nums.push_front(1)
nums.insert_at(3, 99)
print(f"list: {nums.to_list()} size {nums.size}")
nums.remove_at(3)
nums.remove_at(0)
print(f"after removes: {nums.to_list()}")
mid = nums.vals[nums.middle()]
print(f"middle value: {mid}")
nums.reverse()
print(f"reversed: {nums.to_list()}")
merged = merge_sorted(int_list([1, 4, 9, 12]), int_list([2, 3, 10, 15, 20]))
print(f"merged: {merged.to_list()}")
names = LinkedList()
for n in ["ada", "grace", "edsger"]:
    names.push_back(n)
names.push_front("alan")
names.reverse()
print("names reversed: " + ",".join(names.to_list()))
