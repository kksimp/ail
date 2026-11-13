import os, tempfile


def count_endings(text):
    crlf = lf = cr = 0
    i = 0
    while i < len(text):
        c = text[i]
        if c == "\r":
            if i + 1 < len(text) and text[i + 1] == "\n":
                crlf += 1
                i += 1
            else:
                cr += 1
        elif c == "\n":
            lf += 1
        i += 1
    return (crlf, lf, cr)


def convert(text, eol):
    out = []
    i = 0
    while i < len(text):
        c = text[i]
        if c == "\r":
            if i + 1 < len(text) and text[i + 1] == "\n":
                i += 1
            out.append(eol)
        elif c == "\n":
            out.append(eol)
        else:
            out.append(c)
        i += 1
    return "".join(out)


def style(e):
    crlf, lf, cr = e
    total = crlf + lf + cr
    if total == 0:
        return "none"
    if lf == total:
        return "lf"
    if crlf == total:
        return "crlf"
    if cr == total:
        return "cr"
    return "mixed"


def read(p):
    with open(p, "r", newline="") as f:
        return f.read()


def put(p, text):
    with open(p, "w", newline="") as f:
        f.write(text)


def report(d, names):
    print(f"{'file':<11}{'style':<7}{'crlf':>5}{'lf':>4}{'cr':>4}{'bytes':>6}")
    for name in names:
        text = read(os.path.join(d, name))
        e = count_endings(text)
        print(f"{name:<11}{style(e):<7}{e[0]:>5}{e[1]:>4}{e[2]:>4}{len(text.encode()):>6}")


def fix_dir(d, names, eol):
    rewritten = 0
    for name in names:
        p = os.path.join(d, name)
        text = read(p)
        fixed = convert(text, eol)
        if fixed != text:
            put(p, fixed)
            rewritten += 1
    return rewritten


def main():
    d = tempfile.mkdtemp(prefix="corpus_line_endings_fix")
    names = ["unix.txt", "dos.txt", "mixed.txt", "oldmac.txt", "empty.txt"]
    texts = [
        "alpha\nbeta\ngamma\n",
        "one\r\ntwo\r\nthree\r\n",
        "first\r\nsecond\nthird\rfourth",
        "x\ry\rz\r",
        "",
    ]
    for n, t in zip(names, texts):
        put(os.path.join(d, n), t)
    report(d, names)
    n = fix_dir(d, names, "\n")
    print(f"-- to LF: rewrote {n} of {len(names)} files")
    report(d, names)
    pair = ["unix.txt", "dos.txt"]
    m = fix_dir(d, pair, "\r\n")
    print(f"-- to CRLF: rewrote {m} of {len(pair)} files")
    report(d, pair)
    for name in names:
        os.remove(os.path.join(d, name))
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
