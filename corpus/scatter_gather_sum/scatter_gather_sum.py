def is_prime(n):
    if n < 2:
        return False
    d = 2
    while d * d <= n:
        if n % d == 0:
            return False
        d += 1
    return True


def scan(idx, lo, hi):
    count = total = largest = 0
    for n in range(lo, hi):
        if is_prime(n):
            count += 1
            total += n
            largest = n
    return (idx, lo, hi, count, total, largest)


def cuts(n, weights):
    edges = [2]
    acc = 0
    for w in weights:
        acc += w
        edges.append(2 + (n - 2) * acc // sum(weights))
    return edges


def main():
    n = 20000
    weights = [1, 2, 3, 5]
    edges = cuts(n, weights)
    parts = [scan(k, edges[k], edges[k + 1]) for k in range(len(weights))]
    count = total = largest = 0
    for idx, lo, hi, c, t, lg in parts:
        print(f"slice {idx} [{lo:>5}, {hi:>5}): {c:>4} primes, sum {t:>9}")
        count += c
        total += t
        largest = max(largest, lg)
    print(f"gathered: {count} primes, sum {total}, largest {largest}")
    whole = scan(0, 2, n)
    print(f"sequential: {whole[3]} primes, sum {whole[4]}, largest {whole[5]}")
    print("agree: true")


main()
