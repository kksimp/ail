# A three-stage pipeline over bounded queues: one thread produces numbers,
# one squares them, and `main` collects the results. Each stage puts None
# on its output when its input runs dry.

import threading
import queue


def produce(out, n):
    for i in range(1, n + 1):
        out.put(i)
    out.put(None)


def square(input, out):
    while (x := input.get()) is not None:
        out.put(x * x)
    out.put(None)


def main():
    numbers = queue.Queue(2)
    squares = queue.Queue(2)
    stages = [
        threading.Thread(target=produce, args=(numbers, 6)),
        threading.Thread(target=square, args=(numbers, squares)),
    ]
    for t in stages:
        t.start()
    total = 0
    while (s := squares.get()) is not None:
        print(f"square {s}")
        total += s
    for t in stages:
        t.join()
    print(f"total {total}")


main()
