def generate(n):
    state = 12345
    out = []
    for _ in range(n):
        state = (state * 1103515245 + 12345) % 2147483648
        hi = state >> 16
        out.append(hi % 101)
    return out


def buckets(xs, width):
    counts = [0] * (100 // width + 1)
    for x in xs:
        counts[x // width] += 1
    return counts


def bar(n):
    out = ""
    for _ in range(n):
        out = out + "#"
    return out


def percentile(sorted_xs, p):
    n = len(sorted_xs)
    idx = (p * n + 99) // 100 - 1
    return sorted_xs[max(idx, 0)]


def main():
    xs = generate(60)
    counts = buckets(xs, 10)
    peak = max(counts)
    print(f"n = {len(xs)}")
    for i in range(len(counts)):
        lo = i * 10
        hi = min(lo + 9, 100)
        b = bar(counts[i])
        print(f"{lo:>3}-{hi:>3} |{b:<12}| {counts[i]}")
    ordered = sorted(xs)
    print(f"peak bucket count: {peak}")
    print(f"min {ordered[0]}, max {ordered[-1]}, mean {sum(xs) // len(xs)}")
    for p in [25, 50, 75, 90]:
        print(f"p{p}: {percentile(ordered, p)}")


main()
