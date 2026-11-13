def simulate(start, steal):
    n = len(start)
    queues = [list(q) for q in start]
    left = [0] * n
    done = [0] * n
    busy = [0] * n
    remaining = sum(len(q) for q in start)
    steals = 0
    tick = 0
    while remaining > 0 or max(left) > 0:
        for i in range(n):
            if left[i] > 0:
                continue
            if len(queues[i]) > 0:
                left[i] = queues[i].pop(0)
                remaining -= 1
            elif steal:
                victim = -1
                for v in range(n):
                    if v != i and len(queues[v]) > 0:
                        if victim < 0 or len(queues[v]) > len(queues[victim]):
                            victim = v
                if victim >= 0:
                    left[i] = queues[victim].pop()
                    remaining -= 1
                    steals += 1
        for i in range(n):
            if left[i] > 0:
                left[i] -= 1
                busy[i] += 1
                if left[i] == 0:
                    done[i] += 1
        tick += 1
    return tick, steals, done, busy


def show(label, r):
    print(f"{label}: makespan {r[0]}, steals {r[1]}")
    print(f"  jobs done {r[2]}")
    print(f"  busy ticks {r[3]}")


def main():
    work = [[9, 8, 7, 6, 5, 4], [2, 1], [3]]
    total = sum(sum(q) for q in work)
    print(f"total work {total} ticks over {len(work)} workers")
    fixed = simulate(work, False)
    stolen = simulate(work, True)
    show("no stealing", fixed)
    show("stealing", stolen)
    print(f"lower bound {(total + len(work) - 1) // len(work)}")
    print(f"speedup in ticks: {fixed[0] - stolen[0]}")


main()
