# The NumPy twin of f32_chain.ail.
import numpy as np


def main():
    n = 1048576
    x = ((np.arange(n) % 17) * 0.25).astype(np.float32)
    total = 0.0
    for _ in range(20):
        y = (x * x + x) * np.float32(0.5) - x
        total += float(y.astype(np.float64).sum())
    print(total)


main()
