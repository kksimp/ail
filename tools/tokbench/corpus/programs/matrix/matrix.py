def matmul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(len(b))) for j in range(len(b[0]))] for i in range(len(a))]


def transpose(m):
    return [list(col) for col in zip(*m)]


def main():
    a = [[1, 2, 3], [4, 5, 6]]
    b = [[7, 8], [9, 10], [11, 12]]
    c = matmul(a, b)
    print(f"shape {[len(c), len(c[0])]}")
    print(f"product {[x for row in c for x in row]}")
    print(f"transpose {[x for row in transpose(a) for x in row]}")
    print(f"row sums {[sum(row) for row in c]}")


if __name__ == "__main__":
    main()
