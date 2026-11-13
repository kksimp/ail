class QueryError(Exception):
    pass


def make_cond(attr, op, rhs):
    if attr not in ["name", "age", "city"]:
        raise QueryError(f"unknown field {attr}")
    if op not in ["=", "!=", "<", "<=", ">", ">=", "~"]:
        raise QueryError(f"operator {op} does not fit the field")
    if attr == "age":
        if op == "~":
            raise QueryError(f"operator {op} does not fit the field")
        if not (rhs.lstrip("-").isdigit()):
            raise QueryError(f"bad number {rhs}")
    elif op != "=" and op != "!=" and op != "~":
        raise QueryError(f"operator {op} does not fit the field")
    return (attr, op, rhs)


def parse_query(src):
    toks = src.split()
    if not toks:
        raise QueryError("empty query")
    groups = []
    cur = []
    i = 0
    while i < len(toks):
        if i + 3 > len(toks):
            raise QueryError("query ends early")
        cur.append(make_cond(toks[i], toks[i + 1], toks[i + 2]))
        i += 3
        if i < len(toks):
            if toks[i] == "or":
                groups.append(cur)
                cur = []
            elif toks[i] != "and":
                raise QueryError(f"expected and/or, found {toks[i]}")
            i += 1
            if i >= len(toks):
                raise QueryError("query ends early")
    groups.append(cur)
    return groups


def compare(op, l, r):
    if op == "=":
        return l == r
    if op == "!=":
        return l != r
    if op == "<":
        return l < r
    if op == "<=":
        return l <= r
    if op == ">":
        return l > r
    return l >= r


def matches_cond(p, c):
    attr, op, rhs = c
    if attr == "age":
        return compare(op, p[1], int(rhs))
    have = p[0] if attr == "name" else p[2]
    if op == "=":
        return have == rhs
    if op == "!=":
        return have != rhs
    return rhs in have


def matches(p, groups):
    for g in groups:
        all_ok = True
        for c in g:
            if not matches_cond(p, c):
                all_ok = False
        if all_ok:
            return True
    return False


def run_query(people, src):
    try:
        groups = parse_query(src)
    except QueryError as err:
        return f"error: {err}"
    names = [p[0] for p in people if matches(p, groups)]
    return f"{len(names)} -> {', '.join(names)}"


def main():
    people = [
        ("ann", 34, "oslo"),
        ("bob", 27, "bergen"),
        ("cy", 41, "oslo"),
        ("dana", 19, "tromso"),
        ("eli", 52, "bergen"),
    ]
    for q in [
        "city = oslo",
        "age >= 30 and city = bergen",
        "age < 25 or age > 50",
        "name ~ a and age > 30 or city = tromso",
        "age ~ 3",
        "colour = red",
        "age > old",
        "city = oslo and",
        "city = oslo xor age = 3",
        "",
    ]:
        print(f"{q} => {run_query(people, q)}")


main()
