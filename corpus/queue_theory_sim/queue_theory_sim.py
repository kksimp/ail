class SimError(Exception):
    pass


def lcg(s):
    return (s * 1103515245 + 12345) % 2147483648


def make_jobs(n, seed):
    jobs = []
    s = seed
    t = 0
    for i in range(n):
        s = lcg(s)
        t += s // 65536 % 4
        s = lcg(s)
        jobs.append((i, t, s // 65536 % 5 + 1))
    return jobs


def pick_fifo(waiting):
    return 0


def pick_sjf(waiting):
    best = 0
    for i in range(1, len(waiting)):
        if waiting[i][2] < waiting[best][2]:
            best = i
    return best


def pick_lifo(waiting):
    return len(waiting) - 1


def simulate(jobs, servers, pick):
    if servers < 1:
        raise SimError("need at least one server")
    if not jobs:
        raise SimError("no jobs to run")
    free_at = [0] * servers
    waiting = []
    served = total_wait = worst = busy = peak = 0
    nxt = 0
    t = 0
    while served < len(jobs):
        while nxt < len(jobs) and jobs[nxt][1] <= t:
            waiting.append(jobs[nxt])
            nxt += 1
        peak = max(peak, len(waiting))
        for s in range(servers):
            if free_at[s] <= t and waiting:
                job = waiting.pop(pick(waiting))
                waited = t - job[1]
                total_wait += waited
                worst = max(worst, waited)
                busy += job[2]
                free_at[s] = t + job[2]
                served += 1
        t += 1
    return {"served": served, "total_wait": total_wait, "worst": worst,
            "makespan": max(free_at), "busy": busy, "peak": peak}


def fixed2(x):
    return f"{x // 100}.{x % 100:0>2}"


jobs = make_jobs(40, 7)
discs = [("fifo", pick_fifo), ("sjf", pick_sjf), ("lifo", pick_lifo)]
print(f"jobs {len(jobs)}, last arrival {jobs[-1][1]}")
for servers in range(1, 4):
    for label, pk in discs:
        st = simulate(jobs, servers, pk)
        avg = st["total_wait"] * 100 // st["served"]
        util = st["busy"] * 100 // (servers * st["makespan"])
        print(f"{servers} server(s) {label:<5} avg wait {fixed2(avg)} worst {st['worst']} peak {st['peak']} makespan {st['makespan']} util {util}%")
try:
    bad = simulate(jobs, 0, pick_fifo)
except SimError as e:
    print(f"error: {e}")
