def sift_down(xs, start, stop):
    root = start
    swaps = 0
    while 2 * root + 1 < stop:
        child = 2 * root + 1
        if child + 1 < stop and xs[child] < xs[child + 1]:
            child += 1
        if xs[root] >= xs[child]:
            return swaps
        xs[root], xs[child] = xs[child], xs[root]
        swaps += 1
        root = child
    return swaps


def heap_sort(xs):
    n = len(xs)
    swaps = 0
    i = n // 2 - 1
    while i >= 0:
        swaps += sift_down(xs, i, n)
        i -= 1
    last = n - 1
    while last > 0:
        xs[0], xs[last] = xs[last], xs[0]
        swaps += 1
        swaps += sift_down(xs, 0, last)
        last -= 1
    return swaps


def is_sorted(xs):
    for i in range(1, len(xs)):
        if xs[i - 1] > xs[i]:
            return False
    return True


def next_rand(s):
    return (s * 1103515245 + 12345) % 2147483648


def make_data(n, seed):
    s = seed
    out = []
    for _ in range(n):
        s = next_rand(s)
        out.append(s // 65536 % 100)
    return out


def b(v):
    return "true" if v else "false"


def main():
    data = make_data(16, 2024)
    expected = sorted(data)
    print(f"input:  {data}")
    swaps = heap_sort(data)
    print(f"output: {data}")
    print(f"swaps: {swaps}")
    print(f"sorted: {b(is_sorted(data))}")
    print(f"matches built-in sort: {b(data == expected)}")
    tiny = [5, 1]
    heap_sort(tiny)
    print(f"two elements: {tiny}")
    none_at_all = []
    print(f"empty swaps: {heap_sort(none_at_all)}")


main()
