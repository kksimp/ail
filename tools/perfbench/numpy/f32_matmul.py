# The NumPy twin of f32_matmul.ail.
import numpy as np


def main():
    n = 256
    a = ((np.arange(n * n) % 7) * 0.25).astype(np.float32).reshape(n, n)
    b = ((np.arange(n * n) % 5) * 0.5).astype(np.float32).reshape(n, n)
    total = 0.0
    for _ in range(5):
        c = a @ b
        total += float(c.astype(np.float64).sum()) + float(c.max())
    print(total)


main()
