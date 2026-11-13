from collections import deque


class Job:
    def __init__(self, name, arrival, burst):
        self.name = name
        self.arrival = arrival
        self.burst = burst
        self.left = burst
        self.finish = 0

    def turnaround(self):
        return self.finish - self.arrival

    def waiting(self):
        return self.finish - self.arrival - self.burst


def simulate(jobs, quantum):
    work = [Job(j.name, j.arrival, j.burst) for j in jobs]
    n = len(work)
    ready = deque()
    clock = 0
    nxt = 0
    done = 0
    slices = []
    switches = 0
    while done < n:
        while nxt < n and work[nxt].arrival <= clock:
            ready.append(nxt)
            nxt += 1
        if not ready:
            clock = work[nxt].arrival
            continue
        idx = ready.popleft()
        run = min(quantum, work[idx].left)
        slices.append(f"{work[idx].name}x{run}")
        clock += run
        work[idx].left -= run
        while nxt < n and work[nxt].arrival <= clock:
            ready.append(nxt)
            nxt += 1
        if work[idx].left > 0:
            ready.append(idx)
            switches += 1
        else:
            work[idx].finish = clock
            done += 1
    return work, slices, switches


def summarize(label, r):
    work, slices, switches = r
    print(f"{label}: {' '.join(slices)}")
    turn = 0
    wait = 0
    for j in work:
        turn += j.turnaround()
        wait += j.waiting()
    print(f"  turnaround total {turn}, waiting total {wait}, preemptions {switches}")
    finishes = [f"{j.name}@{j.finish}" for j in work]
    print(f"  finish: {' '.join(finishes)}")


jobs = [Job("A", 0, 5), Job("B", 1, 3), Job("C", 2, 8), Job("D", 3, 2), Job("E", 20, 4)]
summarize("q=2", simulate(jobs, 2))
summarize("q=4", simulate(jobs, 4))
summarize("q=100 (first come first served)", simulate(jobs, 100))
