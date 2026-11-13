def load_table(rows):
    table = {}
    for r in rows:
        parts = r.split("=", 1)
        table[parts[0].strip()] = parts[1].strip()
    return table


def is_acronym(word):
    if len(word) < 2:
        return False
    for c in word:
        if not ("A" <= c <= "Z"):
            return False
    return True


def split_punct(token):
    start = 0
    while start < len(token) and not token[start].isalpha():
        start += 1
    stop = len(token)
    while stop > start and not token[stop - 1].isalpha():
        stop -= 1
    return [token[0:start], token[start:stop], token[stop:]]


def expand(text, table):
    seen = set()
    out = []
    for token in text.split():
        parts = split_punct(token)
        core = parts[1]
        if is_acronym(core) and core in table:
            if core not in seen:
                seen.add(core)
                out.append("%s%s (%s)%s" % (parts[0], core, table[core], parts[2]))
            else:
                out.append(token)
        else:
            out.append(token)
    return " ".join(out)


def unknown(text, table):
    found = set()
    for token in text.split():
        core = split_punct(token)[1]
        if is_acronym(core) and core not in table:
            found.add(core)
    return sorted(found)


def main():
    table = load_table(["API = Application Programming Interface", "CPU = Central Processing Unit", "GC = Garbage Collector", "ABI = Application Binary Interface", "IR = Intermediate Representation"])
    text = "The GC pauses the CPU rarely. Each API call uses the ABI, and the IR (lowered by the compiler) feeds the CPU backend; the API stays stable, but the XYZ and QQ parts are new. Does GC, again, matter?"
    print(expand(text, table))
    print("unknown: " + ", ".join(unknown(text, table)))
    print("known: %d" % len(table))


main()
