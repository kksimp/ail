import json


def render(v):
    return json.dumps(v, separators=(",", ":"))


def steps(path):
    out = []
    cur = ""
    for c in path:
        if c == "." or c == "[":
            if cur != "":
                out.append(cur)
            cur = "[" if c == "[" else ""
        elif c == "]":
            out.append(cur + "]")
            cur = ""
        else:
            cur += c
    if cur != "":
        out.append(cur)
    return out


def select(start, step):
    out = []
    for n in start:
        if step.startswith("["):
            if not isinstance(n, list):
                continue
            inner = step[1:-1]
            if inner == "*":
                out.extend(n)
            else:
                if not inner.lstrip("-").isdigit() or inner == "":
                    continue
                i = int(inner)
                if 0 <= i < len(n):
                    out.append(n[i])
        elif isinstance(n, dict):
            for k, v in n.items():
                if step == "*" or k == step:
                    out.append(v)
    return out


def query(root, path):
    cur = [root]
    for st in steps(path):
        cur = select(cur, st)
    return [render(v) for v in cur]


def count_nodes(v):
    if isinstance(v, dict):
        return 1 + sum(count_nodes(x) for x in v.values())
    if isinstance(v, list):
        return 1 + sum(count_nodes(x) for x in v)
    return 1


def sample():
    parts = [
        '{ "store": { "name": "Corner Books", "open": true,',
        '  "books": [',
        '    { "title": "Dune", "price": 12, "tags": ["scifi", "classic"] },',
        '    { "title": "Emma", "price": 9, "tags": [] },',
        '    { "title": "Ulysses", "price": 21, "tags": ["modern"] }',
        "  ],",
        '  "owner": null } }',
    ]
    return "\n".join(parts)


root = json.loads(sample())
print(f"nodes: {count_nodes(root)}")
paths = [
    "store.name", "store.open", "store.books[1].title", "store.books[*].price",
    "store.books[*].tags[0]", "store.books[2]", "store.owner", "store.*",
    "store.books[9]", "store.nope.x",
]
for p in paths:
    found = query(root, p)
    print(f"{p} -> {len(found)}: {' '.join(found)}")
