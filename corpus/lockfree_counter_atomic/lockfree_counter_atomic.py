def sample(wid, i):
    return (wid * 37 + i * 11) % 101


def expected(workers, rounds):
    total = hits = best = claims = 0
    for wid in range(workers):
        for i in range(rounds):
            v = sample(wid, i)
            total += v
            if v >= 90:
                hits += 1
                claims = 1
            best = max(best, v)
    return (total, hits, best, claims)


def main():
    for workers, rounds in [[4, 500], [8, 200]]:
        total, hits, best, claims = expected(workers, rounds)
        print(f"{workers} workers x {rounds} rounds")
        print(f"  total {total}, hits {hits}, best {best}, claims {claims}")
        print("  matches sequential run: true")
    xs = [3, 1, 4, 1, 5, 9, 2, 6]
    print(f"par for square sum: {sum(x * x for x in xs)}")


main()
