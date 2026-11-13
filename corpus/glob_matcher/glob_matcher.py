def find_close(p, open_):
    j = open_ + 1
    if j < len(p) and p[j] == "!":
        j += 1
    j += 1
    while j < len(p):
        if p[j] == "]":
            return j
        j += 1
    return -1


def class_has(p, lo, hi, x):
    neg = p[lo] == "!"
    i = lo + 1 if neg else lo
    found = False
    while i < hi:
        if i + 2 < hi and p[i + 1] == "-":
            if ord(p[i]) <= ord(x) and ord(x) <= ord(p[i + 2]):
                found = True
            i += 3
        else:
            if p[i] == x:
                found = True
            i += 1
    return found != neg


def glob_at(p, pi, t, ti):
    if pi == len(p):
        return ti == len(t)
    c = p[pi]
    if c == "*":
        deep = pi + 1 < len(p) and p[pi + 1] == "*"
        nxt = pi + 2 if deep else pi + 1
        if deep and nxt < len(p) and p[nxt] == "/":
            if glob_at(p, nxt + 1, t, ti):
                return True
        k = ti
        while True:
            if glob_at(p, nxt, t, k):
                return True
            if k >= len(t):
                break
            if t[k] == "/" and not deep:
                break
            k += 1
        return False
    if ti >= len(t):
        return False
    x = t[ti]
    if c == "?":
        return x != "/" and glob_at(p, pi + 1, t, ti + 1)
    if c == "[":
        close = find_close(p, pi)
        if close >= 0:
            return class_has(p, pi + 1, close, x) and glob_at(p, close + 1, t, ti + 1)
    return c == x and glob_at(p, pi + 1, t, ti + 1)


def glob(pat, path):
    return glob_at(list(pat), 0, list(path), 0)


def filter_paths(pat, paths):
    return [s for s in paths if glob(pat, s)]


cases = [
    ("*.ail", "main.ail"),
    ("*.ail", "src/main.ail"),
    ("**/*.ail", "src/lib/main.ail"),
    ("**/*.ail", "main.ail"),
    ("file?.txt", "file7.txt"),
    ("file?.txt", "file10.txt"),
    ("[a-c]*.py", "beta.py"),
    ("[!a-c]*.py", "beta.py"),
    ("data[0-9][0-9].csv", "data42.csv"),
    ("log.[!0-9]", "log.x"),
    ("src/*/test_*.ail", "src/std/test_fs.ail"),
    ("src/*/test_*.ail", "src/std/io/test_fs.ail"),
    ("a[bc", "a[bc"),
    ("*", "a/b"),
    ("**", "a/b"),
    ("", ""),
]
for pat, path in cases:
    verdict = "yes" if glob(pat, path) else "no"
    print(f"{pat:<20} {path:<24} {verdict}")
files = ["a.ail", "b.txt", "src/c.ail", "src/d/e.ail", "src/d/f.txt"]
print("src/**.ail: " + " ".join(filter_paths("src/**.ail", files)))
print("*.*: " + " ".join(filter_paths("*.*", files)))
