import os, tempfile


def weight(i):
    return (i * i + 7) % 97


def save(p, pos, total):
    with open(p, "w", newline="") as f:
        f.write(f"pos={pos}\ntotal={total}\n")


def field(text, key):
    for line in text.splitlines():
        if line.startswith(f"{key}="):
            return int(line[len(key) + 1:])
    return 0


def load(p):
    if not os.path.exists(p):
        return (0, 0)
    with open(p, newline="") as f:
        text = f.read()
    return (field(text, "pos"), field(text, "total"))


def run(p, items, size, max_batches):
    pos, total = load(p)
    print(f"  start at item {pos}, total {total}")
    done = 0
    while pos < items:
        if max_batches >= 0 and done >= max_batches:
            print(f"  killed before batch {done + 1}")
            return (pos, total)
        stop = min(pos + size, items)
        part = 0
        for i in range(pos, stop):
            part += weight(i)
        pos = stop
        total += part
        save(p, pos, total)
        done += 1
        print(f"  batch ends at {pos:>2}, batch sum {part:>3}, total {total:>4}")
    return (pos, total)


def expected(items):
    return sum(weight(i) for i in range(items))


def main():
    d = tempfile.mkdtemp(prefix="corpus_checkpoint_resume_sim")
    p = os.path.join(d, "job.ckpt")
    items = 50
    print("run 1 (interrupted)")
    first = run(p, items, 8, 3)
    print(f"saved: pos {first[0]}, total {first[1]}")
    print("run 2 (resumed)")
    second = run(p, items, 8, -1)
    print(f"final total {second[1]}, expected {expected(items)}")
    print(f"resumed matches uninterrupted: {str(second[1] == expected(items)).lower()}")
    print("run 3 (already finished)")
    third = run(p, items, 8, -1)
    print(f"unchanged: {str(third == second).lower()}")
    os.remove(p)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
