def slow_value(key):
    h = 7
    for c in key:
        h = (h * 31 + ord(c)) % 100003
    return h


def warm_up(shards, cache):
    computed = 0
    skipped = 0
    for keys in shards:
        for key in keys:
            if key in cache:
                skipped += 1
            else:
                cache[key] = slow_value(key)
                computed += 1
    return [computed, skipped]


def lookup_all(queries, cache):
    hits = sum(1 for q in queries if q in cache)
    return [hits, len(queries) - hits]


def main():
    shards = [
        ["alpha", "beta", "gamma", "delta"],
        ["gamma", "epsilon", "alpha"],
        ["zeta", "beta", "eta", "theta"],
    ]
    cache = {}
    first = warm_up(shards, cache)
    print(f"first warm-up: computed {first[0]}, skipped {first[1]}")
    second = warm_up(shards, cache)
    print(f"second warm-up: computed {second[0]}, skipped {second[1]}")
    for key in sorted(cache):
        print(f"{key:<8}{cache[key]:>6}")
    got = lookup_all(["alpha", "omega", "eta", "kappa", "zeta"], cache)
    print(f"lookups: {got[0]} hits, {got[1]} misses")


main()
