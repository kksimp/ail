def sample():
    lines = [
        "; global settings",
        "debug = true",
        "",
        "[server]",
        "host = example.org",
        "port = 8080",
        "# comment",
        "",
        "[client]",
        "retries=3",
        "timeout = 2500 ",
        "bogus line without equals",
    ]
    return "\n".join(lines)


def parse(text):
    out = []
    current = ""
    for raw in text.split("\n"):
        line = raw.strip()
        if line == "" or line.startswith("#") or line.startswith(";"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:len(line) - 1].strip()
            continue
        idx = line.find("=")
        if idx < 0:
            continue
        out.append((current, line[:idx].strip(), line[idx + 1:].strip()))
    return out


def lookup(entries, sec, name):
    for e in entries:
        if e[0] == sec and e[1] == name:
            return e[2]
    return None


def get_int(entries, sec, name, fallback):
    v = lookup(entries, sec, name)
    if v is None:
        return fallback
    try:
        return int(v)
    except ValueError:
        return fallback


def main():
    entries = parse(sample())
    print(f"entries: {len(entries)}")
    for e in entries:
        label = "(global)" if e[0] == "" else e[0]
        print(f"{label}.{e[1]} = {e[2]}")
    print(f"port: {get_int(entries, 'server', 'port', 80)}")
    print(f"retries: {get_int(entries, 'client', 'retries', 0)}")
    print(f"missing: {get_int(entries, 'client', 'nope', -1)}")
    h = lookup(entries, "server", "host")
    print(f"host: {h if h is not None else 'none'}")


main()
