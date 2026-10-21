# The NumPy twin of softmax.ail.
import numpy as np


def softmax(x):
    e = np.exp(x - x.max(axis=-1, keepdims=True))
    return e / e.sum(axis=-1, keepdims=True)


def main():
    n = 512
    x = (((np.arange(n * n) * 7919) % 1009) * 0.01).reshape(n, n)
    f = x.astype(np.float32)
    hits = 0
    mass = 0
    for _ in range(20):
        y = softmax(x)
        z = softmax(f).astype(np.float64)
        hits += int(y.argmax(1).sum()) + int(z.argmax(1).sum())
        mass += int(np.round(y.sum(1) * 1000.0).sum()) + int(np.round(z.sum(1) * 1000.0).sum())
    print(hits)
    print(mass)


main()
