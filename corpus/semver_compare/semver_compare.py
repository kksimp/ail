def split_once(s, sep):
    i = s.find(sep)
    if i < 0:
        return (s, "")
    return (s[:i], s[i + len(sep):])


def to_int(s):
    if s.isdigit() and s.isascii():
        return int(s)
    return None


def parse_ver(text):
    core_all, build = split_once(text, "+")
    core, pre_text = split_once(core_all, "-")
    parts = core.split(".")
    if len(parts) != 3:
        return None
    nums = [to_int(p) for p in parts]
    if any(n is None for n in nums):
        return None
    pre = pre_text.split(".") if pre_text != "" else []
    return {"major": nums[0], "minor": nums[1], "patch": nums[2], "pre": pre, "build": build}


def sign(a, b):
    if a < b:
        return -1
    if a > b:
        return 1
    return 0


def is_num(s):
    if s == "":
        return False
    return all(c.isdigit() for c in s)


def cmp_ident(a, b):
    an = is_num(a)
    bn = is_num(b)
    if an and bn:
        return sign(int(a), int(b))
    if an:
        return -1
    if bn:
        return 1
    if a < b:
        return -1
    if a > b:
        return 1
    return 0


def cmp_ver(x, y):
    for k in ("major", "minor", "patch"):
        if x[k] != y[k]:
            return sign(x[k], y[k])
    if not x["pre"] and not y["pre"]:
        return 0
    if not x["pre"]:
        return 1
    if not y["pre"]:
        return -1
    n = min(len(x["pre"]), len(y["pre"]))
    for i in range(n):
        c = cmp_ident(x["pre"][i], y["pre"][i])
        if c != 0:
            return c
    return sign(len(x["pre"]), len(y["pre"]))


def show_ver(v):
    out = "%d.%d.%d" % (v["major"], v["minor"], v["patch"])
    if v["pre"]:
        out += "-" + ".".join(v["pre"])
    return out


def sort_versions(items):
    vs = list(items)
    for i in range(1, len(vs)):
        j = i
        while j > 0 and cmp_ver(vs[j - 1], vs[j]) > 0:
            vs[j - 1], vs[j] = vs[j], vs[j - 1]
            j -= 1
    return vs


def symbol(c):
    if c < 0:
        return "<"
    if c > 0:
        return ">"
    return "="


def main():
    inputs = [
        "1.0.0", "1.0.0-alpha", "1.0.0-alpha.1", "1.0.0-alpha.beta", "1.0.0-beta",
        "1.0.0-beta.2", "1.0.0-beta.11", "1.0.0-rc.1", "2.1.0+build.5", "1.10.0", "1.2.0", "1.2", "1.x.0",
    ]
    vs = []
    for t in inputs:
        v = parse_ver(t)
        if v is None:
            print("rejected: " + t)
            continue
        vs.append(v)
    for v in sort_versions(vs):
        print(show_ver(v))
    pairs = [("1.0.0", "1.0.0-rc.1"), ("1.0.0-beta.2", "1.0.0-beta.11"), ("2.1.0+a", "2.1.0+b"), ("1.9.9", "1.10.0")]
    for p in pairs:
        x = parse_ver(p[0])
        y = parse_ver(p[1])
        print("%s %s %s" % (p[0], symbol(cmp_ver(x, y)), p[1]))


main()
