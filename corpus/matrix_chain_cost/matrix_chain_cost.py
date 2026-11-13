INF = 1000000000000


def solve(dims):
    n = len(dims) - 1
    cost = [0] * (n * n)
    split = [-1] * (n * n)
    for size in range(2, n + 1):
        for i in range(0, n - size + 1):
            j = i + size - 1
            best = INF
            for k in range(i, j):
                c = cost[i * n + k] + cost[(k + 1) * n + j] + dims[i] * dims[k + 1] * dims[j + 1]
                if c < best:
                    best = c
                    split[i * n + j] = k
            cost[i * n + j] = best
    return n, cost, split


def parenthesize(p, i, j):
    n, cost, split = p
    if i == j:
        return f"A{i + 1}"
    k = split[i * n + j]
    left = parenthesize(p, i, k)
    right = parenthesize(p, k + 1, j)
    return f"({left} {right})"


def naive_cost(dims):
    total = 0
    for i in range(1, len(dims) - 1):
        total += dims[0] * dims[i] * dims[i + 1]
    return total


def report(dims):
    p = solve(dims)
    n = p[0]
    best = p[1][n - 1]
    print(f"dims {'x'.join(str(d) for d in dims)}: {best} multiplications, order {parenthesize(p, 0, n - 1)}, left-to-right {naive_cost(dims)}")


def main():
    report([10, 30, 5, 60])
    report([40, 20, 30, 10, 30])
    report([10, 20, 30, 40, 30])
    report([5, 10, 3, 12, 5, 50, 6])
    report([30, 35, 15, 5, 10, 20, 25])
    report([7, 7, 7, 7, 7])


main()
