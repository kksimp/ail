import os, tempfile


def scan_file(p):
    lines = words = blank = longest = nbytes = 0
    with open(p, "r", newline="") as f:
        for line in f:
            text = line.rstrip()
            lines += 1
            words += len(text.split())
            nbytes += len(line.encode())
            longest = max(longest, len(text.encode()))
            if text == "":
                blank += 1
    return [lines, words, blank, longest, nbytes]


def combine(a, b):
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2], max(a[3], b[3]), a[4] + b[4]]


def extension(name):
    parts = name.split(".")
    if len(parts) < 2:
        return "(none)"
    return parts[-1]


def main():
    d = tempfile.mkdtemp(prefix="corpus_file_stats_report")
    names = ["README.md", "main.ail", "notes.txt", "util.ail", "data.csv", "Makefile"]
    texts = [
        "# Demo\n\nA small demo project.\nIt has two modules.\n",
        "fn main()\n    print(\"hi\")\n\n\nfn helper(x int) int\n    x + 1\n",
        "todo:\n  - write tests   \n  - ship it\n\nlater: refactor everything in sight\n",
        "fn add(a int, b int) int\n    a + b\n",
        "id,name\n1,ann\n2,bob\n3,cy",
        "all:\n\tail build\n",
    ]
    for n, t in zip(names, texts):
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(t)
    print(f"{'file':<11}{'lines':>6}{'words':>6}{'blank':>6}{'longest':>8}{'bytes':>7}")
    total = [0, 0, 0, 0, 0]
    by_lines = {}
    by_count = {}
    busiest = names[0]
    busiest_lines = -1
    for name in names:
        s = scan_file(os.path.join(d, name))
        print(f"{name:<11}{s[0]:>6}{s[1]:>6}{s[2]:>6}{s[3]:>8}{s[4]:>7}")
        total = combine(total, s)
        e = extension(name)
        by_lines[e] = by_lines.get(e, 0) + s[0]
        by_count[e] = by_count.get(e, 0) + 1
        if s[0] > busiest_lines:
            busiest = name
            busiest_lines = s[0]
    print(f"{'total':<11}{total[0]:>6}{total[1]:>6}{total[2]:>6}{total[3]:>8}{total[4]:>7}")
    print(f"most lines: {busiest} ({busiest_lines})")
    for ext in sorted(by_lines):
        print(f"{ext:<7}{by_count[ext]:>2} files{by_lines[ext]:>5} lines")
    for name in names:
        os.remove(os.path.join(d, name))
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
