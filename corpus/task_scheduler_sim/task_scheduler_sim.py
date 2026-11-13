JOBS = [("A", 0, 8), ("B", 1, 4), ("C", 2, 9), ("D", 3, 5), ("E", 30, 2)]


def fcfs(jobs):
    tick = 0
    out = []
    for name, arrival, burst in jobs:
        tick = max(tick, arrival) + burst
        out.append((name, tick, tick - arrival, tick - arrival - burst))
    return out


def round_robin(jobs, quantum):
    n = len(jobs)
    remaining = [j[2] for j in jobs]
    queue = []
    done = []
    tick = 0
    upto = 0
    while len(done) < n:
        while upto < n and jobs[upto][1] <= tick:
            queue.append(upto)
            upto += 1
        if not queue:
            tick = jobs[upto][1]
            continue
        i = queue.pop(0)
        s = min(remaining[i], quantum)
        tick += s
        remaining[i] -= s
        while upto < n and jobs[upto][1] <= tick:
            queue.append(upto)
            upto += 1
        if remaining[i] > 0:
            queue.append(i)
        else:
            name, arrival, burst = jobs[i]
            done.append((name, tick, tick - arrival, tick - arrival - burst))
    return done


def report(title, fs):
    print(title)
    wait = 0
    for name, finish, turn, w in fs:
        print(f"  {name:<4} finish {finish:>3} turnaround {turn:>3} waiting {w:>3}")
        wait += w
    print(f"  average waiting: {wait // len(fs)} (total {wait})")


report("fcfs", fcfs(JOBS))
report("round robin q=3", round_robin(JOBS, 3))
