import queue
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

BATCH = 10


def stream(lines, out):
    for line in lines:
        out.put(line)
    out.put(None)


def tally(batch):
    return (
        len(batch),
        sum(" WARN " in l for l in batch),
        sum(" ERROR " in l for l in batch),
    )


def main():
    path = Path(__file__).resolve().parents[5] / "examples/data/logscan.log"
    ch = queue.Queue(8)
    feed = threading.Thread(target=stream, args=(path.read_text().splitlines(), ch))
    feed.start()
    batches = []
    cur = []
    while (line := ch.get()) is not None:
        cur.append(line)
        if len(cur) == BATCH:
            batches.append(cur)
            cur = []
    feed.join()
    if cur:
        batches.append(cur)
    with ThreadPoolExecutor() as pool:
        tallies = list(pool.map(tally, batches))
    for i, (lines, warns, errors) in enumerate(tallies, 1):
        print(f"batch {i}: {lines} lines, {warns} warn, {errors} error")
    print(f"total {sum(t[0] for t in tallies)} lines, {sum(t[2] for t in tallies)} errors")


if __name__ == "__main__":
    main()
