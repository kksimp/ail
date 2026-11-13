def next_rand(s):
    return (s * 1103515245 + 12345) % 2147483648


def make_customers(n, seed):
    s = seed
    clock = 0
    out = []
    for _ in range(n):
        s = next_rand(s)
        clock += 1 + s // 65536 % 3
        s = next_rand(s)
        out.append((clock, 2 + s // 65536 % 5))
    return out


def simulate(people, tellers, verbose):
    busy_until = [0] * tellers
    line = []
    head = 0
    next_arrival = 0
    st = {"served": 0, "total_wait": 0, "worst_wait": 0, "longest_line": 0, "finished_at": 0}
    t = 0
    while st["served"] < len(people):
        while next_arrival < len(people) and people[next_arrival][0] == t:
            line.append(next_arrival)
            next_arrival += 1
        waiting = len(line) - head
        if waiting > st["longest_line"]:
            st["longest_line"] = waiting
        for k2 in range(tellers):
            if busy_until[k2] <= t and head < len(line):
                who = line[head]
                head += 1
                wait = t - people[who][0]
                busy_until[k2] = t + people[who][1]
                st["served"] += 1
                st["total_wait"] += wait
                st["worst_wait"] = max(st["worst_wait"], wait)
                st["finished_at"] = max(st["finished_at"], busy_until[k2])
                if verbose:
                    print(f"t={t:>2} teller {k2 + 1} takes customer {who + 1} (waited {wait}, needs {people[who][1]})")
        t += 1
    return st


def main():
    people = make_customers(12, 99)
    st = simulate(people, 2, True)
    print(f"served: {st['served']}")
    print(f"total wait: {st['total_wait']}")
    print(f"average wait x10: {st['total_wait'] * 10 // st['served']}")
    print(f"worst wait: {st['worst_wait']}")
    print(f"longest line: {st['longest_line']}")
    print(f"last teller free at: {st['finished_at']}")
    one = simulate(people, 1, False)
    print(f"with one teller: total wait {one['total_wait']}, finished at {one['finished_at']}")


main()
