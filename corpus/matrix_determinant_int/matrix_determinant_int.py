class Mat:
    def __init__(self, dim, cells):
        self.dim = dim
        self.cells = cells

    def at(self, i, j):
        return self.cells[i * self.dim + j]

    def times(self, other):
        out = []
        for i in range(self.dim):
            for j in range(self.dim):
                acc = 0
                for k in range(self.dim):
                    acc += self.at(i, k) * other.at(k, j)
                out.append(acc)
        return Mat(self.dim, out)

    def det_bareiss(self):
        dim = self.dim
        a = list(self.cells)
        sign = 1
        prev = 1
        for k in range(dim - 1):
            if a[k * dim + k] == 0:
                found = -1
                for i in range(k + 1, dim):
                    if found < 0 and a[i * dim + k] != 0:
                        found = i
                if found < 0:
                    return 0
                for j in range(dim):
                    a[k * dim + j], a[found * dim + j] = a[found * dim + j], a[k * dim + j]
                sign = -sign
            for i in range(k + 1, dim):
                for j in range(k + 1, dim):
                    num = a[i * dim + j] * a[k * dim + k] - a[i * dim + k] * a[k * dim + j]
                    a[i * dim + j] = num // prev
            prev = a[k * dim + k]
        return sign * a[dim * dim - 1]

    def minor(self, skip):
        out = []
        for i in range(1, self.dim):
            for j in range(self.dim):
                if j != skip:
                    out.append(self.at(i, j))
        return Mat(self.dim - 1, out)

    def det_cofactor(self):
        if self.dim == 1:
            return self.cells[0]
        total = 0
        for c in range(self.dim):
            term = self.cells[c] * self.minor(c).det_cofactor()
            if c % 2 == 0:
                total += term
            else:
                total -= term
        return total

    def with_column(self, col, repl):
        out = list(self.cells)
        for i in range(self.dim):
            out[i * self.dim + col] = repl[i]
        return Mat(self.dim, out)


def report(label, m):
    print(f"{label}: bareiss {m.det_bareiss()}, cofactor {m.det_cofactor()}")


ident = Mat(4, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1])
tri = Mat(3, [2, -1, 0, -1, 2, -1, 0, -1, 2])
pivot = Mat(3, [0, 2, 1, 1, 0, 3, 4, 1, 0])
flat = Mat(3, [1, 2, 3, 4, 5, 6, 7, 8, 9])
big = Mat(4, [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9, 3])
report("identity", ident)
report("tridiagonal", tri)
report("needs a row swap", pivot)
report("singular", flat)
report("4x4", big)
prod = tri.times(pivot)
print(f"det(A) * det(B) = {tri.det_bareiss() * pivot.det_bareiss()}, det(AB) = {prod.det_bareiss()}")
sys_ = Mat(3, [2, 1, -1, -3, -1, 2, -2, 1, 2])
rhs = [8, -11, -3]
den = sys_.det_bareiss()
print(f"system determinant: {den}")
for c in range(3):
    num = sys_.with_column(c, rhs).det_bareiss()
    print(f"x{c} = {num}/{den} = {num // den}")
