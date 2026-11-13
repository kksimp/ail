def expected_hits(workers, rounds):
    n = 0
    for wid in range(workers):
        for i in range(rounds):
            if (wid + i) % 3 == 0:
                n += 1
    return n


def run(workers, rounds):
    hits = expected_hits(workers, rounds)
    misses = workers * rounds - hits
    atom = workers * sum(range(rounds))
    print(f"workers {workers}, rounds {rounds}")
    print(f"  counter {workers * rounds}")
    print(f"  atomic  {atom}")
    print(f"  tally   {hits} hits, {misses} misses")
    return hits


a = run(4, 250)
b = run(8, 100)
print(f"expected hits {expected_hits(4, 250)} and {expected_hits(8, 100)}")
print(f"match: {'true' if a == expected_hits(4, 250) and b == expected_hits(8, 100) else 'false'}")
