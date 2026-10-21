import math


def sigmoid(x):
    if x >= 0:
        return 1.0 / (1.0 + math.exp(-x))
    e = math.exp(x)
    return e / (1.0 + e)


def fmt(xs):
    return "[" + ", ".join(repr(float(v)) for v in xs) + "]"


def main():
    x = [-1.5, 0.0, 2.0, 3.5]
    print("times two " + fmt([v * 2.0 for v in x]))
    relu = [max(v, 0.0) for v in x]
    print("relu " + fmt(relu))
    print(f"sum of relu {sum(relu)!r}")
    y = [1.0, 2.5]
    print("unary minus " + fmt([-v for v in y]))
    a = [1.0, 2.0, 4.0]
    b = [2.0, 4.0, 8.0]
    print("a / b " + fmt([p / q for p, q in zip(a, b)]))
    print("sigmoid " + fmt([sigmoid(v) for v in [0.0, -1000.0]]))
    print("tanh " + fmt([math.tanh(v) for v in [0.0, 1000.0]]))


if __name__ == "__main__":
    main()
