import os, shutil, tempfile


def count_lines(text):
    total = blank = comment = code = 0
    for line in text.splitlines():
        t = line.strip()
        total += 1
        if t == "":
            blank += 1
        elif t.startswith("#"):
            comment += 1
        else:
            code += 1
    return (total, blank, comment, code)


def row(name, c):
    return f"{name:<10}{c[0]:>6}{c[1]:>6}{c[2]:>6}{c[3]:>6}"


def main():
    d = tempfile.mkdtemp(prefix="corpus_line_counter")
    names = ["main.py", "util.py", "notes.txt", "empty.py"]
    texts = [
        "# entry point\nimport util\n\ndef main():\n    # run it\n    util.go()\n\nmain()\n",
        "def go():\n    print('go')\n\n\n# end\n",
        "todo:\n  - write docs\n\n  - ship\n",
        "",
    ]
    for n, t in zip(names, texts):
        with open(os.path.join(d, n), "w", newline="") as f:
            f.write(t)
    print(f"{'file':<10}{'lines':>6}{'blank':>6}{'cmt':>6}{'code':>6}")
    s = [0, 0, 0, 0]
    for n in sorted(names):
        with open(os.path.join(d, n), newline="") as f:
            c = count_lines(f.read())
        print(row(n, c))
        for i in range(4):
            s[i] += c[i]
    print(row("total", s))
    print(f"code share: {s[3] * 100 // s[0]}%")
    shutil.rmtree(d)


main()
