SCHEMA = [
    ("name", "text", "", True),
    ("port", "int", "1-65535", True),
    ("mode", "choice", "fast|slow|safe", False),
    ("debug", "bool", "", False),
    ("workers", "int", "1-64", False),
]


def check_value(rule, value):
    key, kind, arg, req = rule
    if value == "":
        return "empty value"
    if kind == "int":
        try:
            n = int(value)
        except ValueError:
            return f"not an integer: {value}"
        lo, hi = (int(x) for x in arg.split("-"))
        if n < lo or n > hi:
            return f"{n} outside {lo}..{hi}"
    elif kind == "bool":
        if value != "true" and value != "false":
            return f"not a bool: {value}"
    elif kind == "choice":
        if value not in arg.split("|"):
            return f"{value} not one of {arg.replace('|', ', ')}"
    return ""


def validate(text):
    problems = []
    seen = set()
    lines = text.split("\n")[:-1]
    for i, raw in enumerate(lines):
        line = raw.strip()
        if line == "" or line.startswith("#"):
            continue
        at = i + 1
        if "=" not in line:
            problems.append(f"line {at}: expected key = value")
            continue
        eq = line.index("=")
        key = line[:eq].strip()
        value = line[eq + 1:].strip()
        if key in seen:
            problems.append(f"line {at}: duplicate key {key}")
            continue
        seen.add(key)
        matching = [r for r in SCHEMA if r[0] == key]
        if not matching:
            problems.append(f"line {at}: unknown key {key}")
            continue
        msg = check_value(matching[0], value)
        if msg != "":
            problems.append(f"line {at}: {key}: {msg}")
    for r in SCHEMA:
        if r[3] and r[0] not in seen:
            problems.append(f"missing required key {r[0]}")
    return problems


def main():
    names = ["good.conf", "bad.conf", "sloppy.conf"]
    texts = [
        "# web service\nname = api\nport = 8080\nmode = fast\ndebug = false\n",
        "name = web\nport = 70000\nmode = turbo\nworkers = many\ndebug = yes\n",
        "port = 80\nport = 81\ncolour = red\nthis line is wrong\nname =\n",
    ]
    worst = 0
    for n, t in zip(names, texts):
        problems = validate(t)
        code = 0 if not problems else 1
        worst = max(worst, code)
        print(f"{n}: {len(problems)} problems, exit {code}")
        for p in problems:
            print(f"  {p}")
    print(f"overall exit code {worst}")


main()
