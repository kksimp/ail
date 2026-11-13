import os, shutil, tempfile


def write_log(p, count):
    with open(p, "w", newline="") as w:
        for i in range(1, count + 1):
            w.write(f"line {i:03} value={i * 7 % 13}\n")


def tail(p, n):
    window = []
    with open(p, newline="") as r:
        for line in r:
            window.append(line.rstrip())
            if len(window) > n:
                window.pop(0)
    return window


def longest_line(p):
    best = 0
    with open(p, newline="") as r:
        for line in r:
            best = max(best, len(line.rstrip().encode()))
    return best


def main():
    d = tempfile.mkdtemp(prefix="corpus_tail_file")
    p = os.path.join(d, "app.log")
    write_log(p, 20)
    print(f"lines: {len(tail(p, 1000))}, bytes: {os.path.getsize(p)}, longest: {longest_line(p)}")
    print("last 5 lines:")
    for l in tail(p, 5):
        print(f"  {l}")
    short = tail(p, 100)
    print(f"asking for 100 gives {len(short)} lines, first: {short[0]}")
    none_left = tail(p, 0)
    print(f"asking for 0 gives {len(none_left)} lines")
    os.remove(p)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
