import os, tempfile


def log_name(base, n):
    if n == 0:
        return base
    return f"{base}.{n}"


def current_size(p):
    if not os.path.exists(p):
        return 0
    return os.path.getsize(p)


def rotate(d, base, keep):
    oldest = os.path.join(d, log_name(base, keep - 1))
    if os.path.exists(oldest):
        os.remove(oldest)
    for k in range(0, keep - 1):
        j = keep - 2 - k
        src = os.path.join(d, log_name(base, j))
        if os.path.exists(src):
            os.rename(src, os.path.join(d, log_name(base, j + 1)))


def append(d, base, line, max_bytes, keep):
    p = os.path.join(d, base)
    size = current_size(p)
    rotated = False
    if size > 0 and size + len(line.encode()) + 1 > max_bytes:
        rotate(d, base, keep)
        rotated = True
    with open(p, "a", newline="") as f:
        f.write(line + "\n")
    return rotated


def show_state(d):
    for name in sorted(os.listdir(d)):
        size = os.path.getsize(os.path.join(d, name))
        print(f"  {name:<10}{size:>3} bytes")


def read_lines(p):
    with open(p, newline="") as f:
        return f.read().splitlines()


def main():
    d = tempfile.mkdtemp(prefix="corpus_log_rotate_sim")
    for i in range(1, 15):
        rotated = append(d, "app.log", f"event {i:02} ok", 50, 3)
        if rotated:
            print(f"rotated before event {i}")
    print("files after 14 events:")
    show_state(d)
    oldest = read_lines(os.path.join(d, "app.log.2"))
    print(f"oldest file starts with: {oldest[0]}")
    newest = read_lines(os.path.join(d, "app.log"))
    print(f"current file holds {len(newest)} lines, last: {newest[-1]}")
    for name in os.listdir(d):
        os.remove(os.path.join(d, name))
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
