def run(start):
    n = start
    steps = 0
    peak = start
    while n != 1:
        if n % 2 == 0:
            n = n // 2
        else:
            n = 3 * n + 1
        steps += 1
        if n > peak:
            peak = n
    return steps, peak


def bar(count):
    return "#" * (count // 20)


def main():
    for n in range(1, 13):
        steps, peak = run(n)
        print(f"{n:>3}: {steps:>3} steps, peak {peak}")
    best_n = 1
    best_steps = 0
    top_n = 1
    top_peak = 1
    total = 0
    buckets = [0, 0, 0, 0, 0]
    for m in range(1, 1000):
        steps, peak = run(m)
        total += steps
        if steps > best_steps:
            best_steps = steps
            best_n = m
        if peak > top_peak:
            top_peak = peak
            top_n = m
        slot = steps // 50
        buckets[slot] = buckets[slot] + 1
    print(f"longest chain below 1000: {best_n} with {best_steps} steps")
    print(f"highest peak below 1000: {top_peak} (start {top_n})")
    print(f"mean steps: {total // 999}")
    for b in range(len(buckets)):
        lo = b * 50
        hi = lo + 49
        count = buckets[b]
        print(f"{lo:>3}-{hi:>3}: {count:>3} {bar(count)}")


main()
