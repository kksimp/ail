import os, tempfile, shutil

BATCHES = [
    (True, ["boot start", "boot done"]),
    (False, ["GET /a 200"]),
    (False, []),
    (False, ["GET /b 404", "GET /c 200", "POST /d 201"]),
    (True, ["rotated: new log"]),
    (False, ["GET /e 200"]),
]


def text_of(lines):
    return "".join(l + "\n" for l in lines)


def apply(p, rotate, lines):
    if rotate:
        with open(p, "w", newline="") as f:
            f.write(text_of(lines))
    elif lines:
        with open(p, "a", newline="") as f:
            f.write(text_of(lines))


def poll(p, offset):
    start = offset
    if os.path.getsize(p) < offset:
        start = 0
    fresh = []
    with open(p, "rb") as r:
        r.seek(start)
        while True:
            line = r.readline()
            if not line:
                break
            fresh.append(line.decode().rstrip())
        end = r.tell()
    return fresh, end


def main():
    d = tempfile.mkdtemp(prefix="corpus_file_tail_follow_sim")
    p = os.path.join(d, "follow.log")
    offset = 0
    seen = 0
    for i, (rotate, lines) in enumerate(BATCHES):
        tick = i + 1
        apply(p, rotate, lines)
        fresh, end = poll(p, offset)
        if end < offset:
            print(f"poll {tick}: file was rotated")
        elif not fresh:
            print(f"poll {tick}: nothing new")
        for l in fresh:
            print(f"poll {tick}: {l}")
            seen += 1
        offset = end
    print(f"{seen} lines seen, final offset {offset}")
    shutil.rmtree(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
