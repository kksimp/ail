def failure_table(p):
    table = [0]
    k = 0
    for i in range(1, len(p)):
        while k > 0 and p[i] != p[k]:
            k = table[k - 1]
        if p[i] == p[k]:
            k += 1
        table.append(k)
    return table


def find_all(text, pattern):
    t = list(text)
    p = list(pattern)
    if not p:
        return []
    table = failure_table(p)
    hits = []
    k = 0
    for i in range(len(t)):
        while k > 0 and t[i] != p[k]:
            k = table[k - 1]
        if t[i] == p[k]:
            k += 1
        if k == len(p):
            hits.append(i - k + 1)
            k = table[k - 1]
    return hits


def naive_find_all(text, pattern):
    t = list(text)
    p = list(pattern)
    hits = []
    if not p or len(p) > len(t):
        return hits
    for i in range(len(t) - len(p) + 1):
        if t[i:i + len(p)] == p:
            hits.append(i)
    return hits


def j(xs):
    return " ".join(str(x) for x in xs)


def main():
    text = "abracadabra abracadabra, aaaa banana bandana"
    print(f"text: {text}")
    for pat in ["abra", "aa", "ana", "bandana", "xyz", "abracadabra abracadabra"]:
        table = failure_table(list(pat))
        hits = find_all(text, pat)
        print(f"{pat}: table [{j(table)}] hits [{j(hits)}] count {len(hits)}")
    print("naive agrees: " + ("true" if naive_find_all(text, "ana") == find_all(text, "ana") else "false"))


main()
