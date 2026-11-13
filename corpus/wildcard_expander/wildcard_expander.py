def matching(cs, open_at):
    depth = 0
    i = open_at
    while i < len(cs):
        if cs[i] == "{":
            depth += 1
        elif cs[i] == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def split_top(body):
    parts = []
    cur = ""
    depth = 0
    for c in body:
        if c == "," and depth == 0:
            parts.append(cur)
            cur = ""
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
        cur = f"{cur}{c}"
    parts.append(cur)
    return parts


def to_int(s):
    t = s[1:] if s.startswith("-") else s
    if t == "" or not t.isdigit():
        return None
    return int(s)


def range_items(body):
    parts = body.split("..")
    if len(parts) != 2:
        return []
    lo = to_int(parts[0])
    hi = to_int(parts[1])
    if lo is None or hi is None:
        return []
    padded = (len(parts[0]) > 1 and parts[0].startswith("0")) or (len(parts[1]) > 1 and parts[1].startswith("0"))
    width = max(len(parts[0]), len(parts[1])) if padded else 0
    out = []
    n = lo
    while (lo <= hi and n <= hi) or (lo > hi and n >= hi):
        out.append(str(n).rjust(width, "0"))
        n = n + 1 if lo <= hi else n - 1
    return out


def expand(s):
    cs = s
    i = 0
    while i < len(cs):
        if cs[i] == "{":
            shut = matching(cs, i)
            if shut > 0:
                body = cs[i + 1:shut]
                alts = split_top(body)
                if len(alts) < 2:
                    alts = range_items(body)
                if len(alts) > 0:
                    pre = cs[:i]
                    post = cs[shut + 1:]
                    out = []
                    for a in alts:
                        out.extend(expand(f"{pre}{a}{post}"))
                    return out
        i += 1
    return [s]


def wild(p, pi, t, ti):
    if pi == len(p):
        return ti == len(t)
    if p[pi] == "*":
        k = ti
        while k <= len(t):
            if wild(p, pi + 1, t, k):
                return True
            k += 1
        return False
    if ti >= len(t):
        return False
    return (p[pi] == "?" or p[pi] == t[ti]) and wild(p, pi + 1, t, ti + 1)


def pick_files(pattern, files):
    hits = []
    for pat in expand(pattern):
        for f in files:
            if wild(pat, 0, f, 0) and f not in hits:
                hits.append(f)
    return hits


def main():
    for p in ["file{1,2,3}.txt", "{a,b}{x,y}", "img{08..11}.png", "n{3..1}", "src/{lib,bin/{cli,srv}}.ail", "{solo}", "a{b,c", "pre{,s}"]:
        print(f"{p} => {' '.join(expand(p))}")
    files = ["main.c", "main.h", "util.c", "util.h", "util.o", "README.md", "a1.txt", "a2.txt", "a10.txt"]
    for q in ["*.{c,h}", "a?.txt", "{util,main}.*", "*.{py,rs}"]:
        print(f"{q}: {' '.join(pick_files(q, files))}")


main()
