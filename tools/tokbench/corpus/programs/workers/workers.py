# Fan-out, fan-in: four workers take jobs from one queue and send results
# to another. The last worker to finish puts a sentinel, so `main` reads
# results until it sees it.

import threading
import queue
from dataclasses import dataclass


@dataclass
class Done:
    job: int
    worker: int
    steps: int


def steps(n):
    count = 0
    while n != 1:
        n = n // 2 if n % 2 == 0 else 3 * n + 1
        count += 1
    return count


def feed(jobs, n):
    for i in range(1, n + 1):
        jobs.put(i)
    for _ in range(4):
        jobs.put(None)


def work(id, jobs, results, active, lock):
    while (n := jobs.get()) is not None:
        results.put(Done(n, id, steps(n)))
    with lock:
        active[0] -= 1
        if active[0] == 0:
            results.put(None)


def main():
    jobs = queue.Queue(4)
    results = queue.Queue(4)
    active = [4]
    lock = threading.Lock()
    threads = [threading.Thread(target=feed, args=(jobs, 12))]
    for id in range(1, 5):
        threads.append(threading.Thread(target=work, args=(id, jobs, results, active, lock)))
    for t in threads:
        t.start()
    all = list(iter(results.get, None))
    for t in threads:
        t.join()
    for d in sorted(all, key=lambda d: d.job):
        print(f"job {d.job}: {d.steps} steps")
    print(f"longest {max(all, key=lambda d: d.steps).job}")


main()
