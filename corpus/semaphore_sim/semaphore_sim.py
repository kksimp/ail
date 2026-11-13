def simulate(jobs, permits):
    n = len(jobs)
    free = permits
    waiting = []
    ends = [-1] * n
    running = []
    log = []
    peak = 0
    total_wait = 0
    finished = 0
    tick = 0
    while finished < n:
        for i in range(n):
            if ends[i] == tick:
                free += 1
                finished += 1
                running = [r for r in running if r != i]
                log.append(f"t={tick:>2} release {jobs[i][0]} (free {free})")
        for i in range(n):
            if jobs[i][1] == tick:
                waiting.append(i)
                log.append(f"t={tick:>2} arrive  {jobs[i][0]}")
        while free > 0 and waiting:
            i = waiting.pop(0)
            free -= 1
            ends[i] = tick + jobs[i][2]
            running.append(i)
            total_wait += tick - jobs[i][1]
            peak = max(peak, len(running))
            log.append(f"t={tick:>2} acquire {jobs[i][0]} (waited {tick - jobs[i][1]})")
        tick += 1
    return log, peak, total_wait


JOBS = [("j1", 0, 4), ("j2", 1, 3), ("j3", 1, 2), ("j4", 2, 2), ("j5", 9, 1)]
log, peak, total_wait = simulate(JOBS, 2)
for line in log:
    print(line)
print(f"peak holders {peak}, total wait {total_wait}")
_, wpeak, wwait = simulate(JOBS, 4)
print(f"with 4 permits: peak {wpeak}, total wait {wwait}")
