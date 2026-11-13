def parse(words, specs):
    flags = {}
    opts = {}
    poss = []
    for s in specs:
        if s[0] == "flag":
            flags[s[1]] = s[2]
        elif s[0] == "opt":
            opts[s[1]] = (s[2], s[3])
        elif s[0] == "pos":
            poss.append(s[1])
    has_rest = any(s[0] == "rest" for s in specs)
    short = {}
    for s in specs:
        if s[0] in ("flag", "opt") and s[2] != "":
            short[s[2]] = s[1]
    got_flags = set()
    vals = {}
    for s in specs:
        if s[0] == "opt":
            vals[s[1]] = s[3]
    words_pos = []
    i = 0
    only_pos = False
    while i < len(words):
        w = words[i]
        i += 1
        if only_pos or w == "-" or not w.startswith("-"):
            words_pos.append(w)
            continue
        if w == "--":
            only_pos = True
            continue
        if w.startswith("--"):
            body = w[2:]
            if "=" in body:
                name, val = body.split("=", 1)
                has_val = True
            else:
                name, val, has_val = body, None, False
        else:
            name = short.get(w[1:])
            val, has_val = None, False
            if name is None:
                return None
        if name in flags:
            if has_val:
                return None
            got_flags.add(name)
        elif name in opts:
            if not has_val:
                if i >= len(words):
                    return None
                val = words[i]
                i += 1
            vals[name] = val
        else:
            return None
    if len(words_pos) < len(poss):
        return None
    if len(words_pos) > len(poss) and not has_rest:
        return None
    for n, w in zip(poss, words_pos):
        vals[n] = w
    return {"flags": got_flags, "vals": vals, "rest": words_pos[len(poss):]}


def to_int(s):
    t = s.strip()
    if t.lstrip("-").isdigit() and t != "":
        return int(t)
    return 0


def b(x):
    return "true" if x else "false"


def convert(words):
    spec = [
        ("flag", "verbose", "v"),
        ("opt", "out", "o", "a.txt"),
        ("opt", "level", "l", "5"),
        ("pos", "input"),
    ]
    a = parse(words, spec)
    if a is None:
        return "error"
    level = to_int(a["vals"]["level"])
    return f"in={a['vals']['input']} out={a['vals']['out']} level={level} verbose={b('verbose' in a['flags'])}"


def bundle(words):
    spec = [("flag", "dry", "n"), ("pos", "name"), ("rest", "files")]
    a = parse(words, spec)
    if a is None:
        return "error"
    return f"name={a['vals']['name']} dry={b('dry' in a['flags'])} files={len(a['rest'])} [{','.join(a['rest'])}]"


def main():
    print("== convert ==")
    runs = [
        ["in.txt"],
        ["-v", "in.txt"],
        ["--out", "b.bin", "in.txt"],
        ["--out=c.bin", "--level=9", "in.txt"],
        ["in.txt", "-l", "2", "-v"],
        ["--", "-weird"],
        ["--nope", "in.txt"],
        ["in.txt", "--out"],
        ["--verbose=yes", "in.txt"],
        [],
        ["a.txt", "b.txt"],
    ]
    for i, r in enumerate(runs):
        print(f"{i + 1}: {convert(r)}")
    print("== bundle ==")
    bundles = [
        ["site"],
        ["site", "a.html", "b.css"],
        ["-n", "site", "a.html"],
        ["site", "--", "-x", "-y"],
        ["--dry"],
    ]
    for i, r in enumerate(bundles):
        print(f"{i + 1}: {bundle(r)}")


main()
