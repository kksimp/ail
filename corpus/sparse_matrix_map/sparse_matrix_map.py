class Sparse:
    def __init__(self, rows, cols):
        self.rows = rows
        self.cols = cols
        self.cells = {}

    def put(self, r, c, v):
        if v == 0:
            self.cells.pop((r, c), None)
        else:
            self.cells[(r, c)] = v

    def at(self, r, c):
        return self.cells.get((r, c), 0)

    def nonzero(self):
        return len(self.cells)

    def plus(self, other):
        out = Sparse(self.rows, self.cols)
        for i in range(self.rows):
            for j in range(self.cols):
                out.put(i, j, self.at(i, j) + other.at(i, j))
        return out

    def transpose(self):
        out = Sparse(self.cols, self.rows)
        for i in range(self.rows):
            for j in range(self.cols):
                out.put(j, i, self.at(i, j))
        return out

    def times(self, other):
        out = Sparse(self.rows, other.cols)
        for i in range(self.rows):
            for k in range(self.cols):
                a = self.at(i, k)
                if a != 0:
                    for j in range(other.cols):
                        out.put(i, j, out.at(i, j) + a * other.at(k, j))
        return out

    def dense(self):
        return ["".join(f"{self.at(i, j):>3}" for j in range(self.cols)) for i in range(self.rows)]


a = Sparse(3, 4)
a.put(0, 0, 2)
a.put(0, 3, -1)
a.put(1, 1, 5)
a.put(2, 2, 3)
a.put(2, 0, 4)
b = Sparse(4, 3)
b.put(0, 1, 1)
b.put(1, 0, 2)
b.put(2, 2, 7)
b.put(3, 0, 6)
print(f"A has {a.nonzero()} of 12 cells")
for line in a.dense():
    print(line)
print("A transposed:")
for line in a.transpose().dense():
    print(line)
print("A x B:")
prod = a.times(b)
for line in prod.dense():
    print(line)
print(f"product nonzero: {prod.nonzero()}")
c = Sparse(3, 4)
c.put(0, 3, 1)
c.put(1, 1, -5)
s = a.plus(c)
print(f"A + C has {s.nonzero()} cells, (0,3) = {s.at(0, 3)}")
