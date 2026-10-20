# The work of tensor.ail in Python without NumPy.
def main():
    n = 160
    xs = [(i % 17) * 0.25 for i in range(n * n)]
    ys = [(i % 13) * 0.5 for i in range(n * n)]
    c = [0.0] * (n * n)
    for i in range(n):
        for p in range(n):
            x = xs[i * n + p]
            row = i * n
            for j in range(n):
                c[row + j] += x * ys[p * n + j]
    total = sum(c)
    total += sum((xs[i] + ys[i]) * xs[i] for i in range(n * n))
    total += sum(xs[i] * ys[i] for i in range(n * n))
    print(total)

main()
