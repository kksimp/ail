import re


def to_int(s):
    return int(s) if re.fullmatch(r"-?[0-9]+", s) else None


def partial(s):
    nums = []
    for p in s.split("."):
        if p == "x" or p == "*":
            break
        n = to_int(p)
        if n is None:
            return None
        nums.append(n)
    if len(nums) > 3:
        return None
    return nums


def parse_ver(s):
    nums = partial(s)
    if nums is None or len(nums) != 3:
        return None
    return tuple(nums)


def show(v):
    return f"{v[0]}.{v[1]}.{v[2]}"


def pair(lo, hi):
    return [(">=", lo), ("<", hi)]


def expand(term):
    if term.startswith("^"):
        v = parse_ver(term[1:])
        if v is None:
            return None
        if v[0] > 0:
            return pair(v, (v[0] + 1, 0, 0))
        if v[1] > 0:
            return pair(v, (0, v[1] + 1, 0))
        return pair(v, (0, 0, v[2] + 1))
    if term.startswith("~"):
        v = parse_ver(term[1:])
        if v is None:
            return None
        return pair(v, (v[0], v[1] + 1, 0))
    for op in [">=", "<=", ">", "<", "="]:
        if term.startswith(op):
            v = parse_ver(term[len(op):])
            if v is None:
                return None
            return [(op, v)]
    nums = partial(term)
    if nums is None:
        return None
    if len(nums) == 3:
        v = parse_ver(term)
        if v is None:
            return None
        return [("=", v)]
    if len(nums) == 2:
        return pair((nums[0], nums[1], 0), (nums[0], nums[1] + 1, 0))
    if len(nums) == 1:
        return pair((nums[0], 0, 0), (nums[0] + 1, 0, 0))
    return []


def compile_range(text):
    alts = []
    for alt in text.split("||"):
        cmps = []
        for t in alt.split():
            e = expand(t)
            if e is None:
                return None
            cmps.extend(e)
        alts.append(cmps)
    return alts


def holds(c, v):
    op, w = c
    if op == ">=":
        return v >= w
    if op == "<=":
        return v <= w
    if op == ">":
        return v > w
    if op == "<":
        return v < w
    return v == w


def satisfies(alts, v):
    for cmps in alts:
        if all(holds(c, v) for c in cmps):
            return True
    return False


inputs = ["0.9.9", "1.0.0", "1.2.3", "1.4.0", "1.9.9", "2.0.0", "2.3.1", "0.2.5", "0.0.3"]
versions = []
for t in inputs:
    parsed = parse_ver(t)
    if parsed is None:
        continue
    versions.append(parsed)
ranges = [">=1.2.0 <2.0.0", "^1.2.3", "^0.2.3", "^0.0.3", "~1.2.3", "1.x", "2", "=1.4.0", "<1.0.0 || >=2.3.0", "^1.x", "*"]
for r in ranges:
    alts = compile_range(r)
    if alts is None:
        print(f"{r:<20} invalid")
        continue
    hits = [show(x) for x in versions if satisfies(alts, x)]
    print(f"{r:<20} {' '.join(hits)}")
