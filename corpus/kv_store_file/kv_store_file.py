import os, tempfile


def load(p):
    m = {}
    if not os.path.exists(p):
        return m
    with open(p, newline="") as f:
        for line in f.read().splitlines():
            parts = line.split("=", 1)
            if len(parts) == 2:
                m[parts[0]] = parts[1]
    return m


def save(p, m):
    out = [f"{k}={m[k]}" for k in sorted(m)]
    with open(p, "w", newline="") as f:
        f.write("\n".join(out) + "\n")


def apply(line, store):
    parts = line.split(" ", 2)
    op = parts[0]
    if op == "set" and len(parts) == 3:
        store[parts[1]] = parts[2]
        return "OK"
    if op == "get" and len(parts) == 2:
        return store.get(parts[1], "(nil)")
    if op == "del" and len(parts) == 2:
        if parts[1] in store:
            del store[parts[1]]
            return "1"
        return "0"
    if op == "incr" and len(parts) == 2:
        n = int(store.get(parts[1], "0"))
        store[parts[1]] = str(n + 1)
        return str(n + 1)
    if op == "count":
        return str(len(store))
    return "ERR unknown command"


def session(p, cmds):
    store = load(p)
    for c in cmds:
        print(f"> {c}")
        print(apply(c, store))
    save(p, store)


def show(p):
    with open(p, newline="") as f:
        for l in f.read().splitlines():
            print(l)


def main():
    d = tempfile.mkdtemp(prefix="corpus_kv_store_file")
    p = os.path.join(d, "store.db")
    print("-- session 1")
    session(p, ["set name ada", "set url http://x.org/?a=b", "incr visits", "incr visits", "get name", "get missing", "frobnicate"])
    print("-- file after session 1")
    show(p)
    print("-- session 2")
    session(p, ["get visits", "incr visits", "del name", "del name", "set city rome", "count"])
    print("-- file after session 2")
    show(p)
    os.remove(p)
    os.rmdir(d)


main()
