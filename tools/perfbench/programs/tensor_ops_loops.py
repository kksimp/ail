# The work of tensor_ops.ail in Python without NumPy (the twin of tensor_ops_loops.ail too).
def main():
    n = 256
    a = [(i % 17) * 0.25 for i in range(n * n)]
    b = [(i % 13) * 0.5 for i in range(n * n)]
    total = 0.0
    for _ in range(10):
        d = [(a[i] + b[i]) * a[i] - b[i] * 0.5 for i in range(n * n)]
        t = [0.0] * (n * n)
        for i in range(n):
            for j in range(n):
                t[j * n + i] = d[i * n + j]
        total += sum(t)
        total += sum(max(a[i * n + j] for i in range(n)) for j in range(n))
        total += sum(sum(b[i * n:(i + 1) * n]) / n for i in range(n))
        total += max(sum(t[i * n:(i + 1) * n]) for i in range(n))
    print(total)

main()
