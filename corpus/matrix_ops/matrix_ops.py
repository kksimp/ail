def identity(n):
    return [[1 if i == j else 0 for j in range(n)] for i in range(n)]


def mat_mul(a, b, m):
    n = len(a)
    out = []
    for i in range(n):
        row = []
        for j in range(n):
            acc = 0
            for k in range(n):
                acc += a[i][k] * b[k][j]
            row.append(acc % m)
        out.append(row)
    return out


def mat_pow(base, e, m):
    result = identity(len(base))
    sq = base
    left = e
    while left > 0:
        if left % 2 == 1:
            result = mat_mul(result, sq, m)
        sq = mat_mul(sq, sq, m)
        left //= 2
    return result


def transpose(a):
    n = len(a)
    return [[a[j][i] for j in range(n)] for i in range(n)]


def trace(a):
    return sum(a[i][i] for i in range(len(a)))


def det3(a):
    return (a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]))


def show(label, a):
    print(f"{label}:")
    for row in a:
        print("  " + " ".join(str(x) for x in row))


def main():
    a = [[2, 0, 1], [1, 3, 2], [0, 1, 4]]
    b = [[1, 2, 0], [0, 1, 1], [3, 0, 1]]
    big = 1000003
    show("a * b", mat_mul(a, b, big))
    show("transpose of a", transpose(a))
    print(f"trace of a: {trace(a)}")
    print(f"det of a: {det3(a)}")
    print(f"det of b: {det3(b)}")
    fib = [[1, 1], [1, 0]]
    for n in [1, 10, 30, 90]:
        f = mat_pow(fib, n, big)
        print(f"fib({n}) mod {big} = {f[0][1]}")
    walks = [[0, 1, 1], [1, 0, 1], [1, 1, 0]]
    w = mat_pow(walks, 6, big)
    print(f"walks of length 6 from node 0 to node 0: {w[0][0]}")


main()
