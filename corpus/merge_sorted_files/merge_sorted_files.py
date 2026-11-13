import os, tempfile


def write_run(p, xs):
    with open(p, "w", newline="") as f:
        for x in xs:
            f.write(f"{x}\n")


def read_run(p):
    with open(p, newline="") as f:
        return [int(l) for l in f.read().split("\n") if l != ""]


def merge_k(runs):
    pos = [0] * len(runs)
    out = []
    while True:
        best = -1
        for k in range(len(runs)):
            if pos[k] < len(runs[k]):
                if best < 0 or runs[k][pos[k]] < runs[best][pos[best]]:
                    best = k
        if best < 0:
            break
        out.append(runs[best][pos[best]])
        pos[best] += 1
    return out


def dedupe(xs):
    out = []
    for x in xs:
        if not out or out[-1] != x:
            out.append(x)
    return out


def is_sorted(xs):
    return all(xs[i - 1] <= xs[i] for i in range(1, len(xs)))


def make_run(step, offset, n):
    return [i * step + offset for i in range(n)]


def main():
    d = tempfile.mkdtemp(prefix="corpus_merge_sorted_files")
    names = ["run0.txt", "run1.txt", "run2.txt"]
    runs = [make_run(3, 0, 8), make_run(5, 2, 6), make_run(7, 1, 5)]
    loaded = []
    for i in range(len(names)):
        p = os.path.join(d, names[i])
        write_run(p, runs[i])
        loaded.append(read_run(p))
        print(f"{names[i]}: {len(loaded[i])} numbers, first {loaded[i][0]}, last {loaded[i][-1]}")
    merged = merge_k(loaded)
    out_path = os.path.join(d, "merged.txt")
    write_run(out_path, merged)
    back = read_run(out_path)
    print(f"merged: {len(back)} numbers, sorted {str(is_sorted(back)).lower()}")
    print(f"first 10: {back[:10]}")
    print(f"last 3: {back[-3:]}")
    unique = dedupe(back)
    print(f"unique: {len(unique)}, dropped {len(back) - len(unique)}")
    print(f"round trip intact: {str(back == merged).lower()}")
    for n in names:
        os.remove(os.path.join(d, n))
    os.remove(out_path)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
