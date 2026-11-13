def distance(a, b):
    xs = list(a)
    ys = list(b)
    prev = list(range(len(ys) + 1))
    for i in range(1, len(xs) + 1):
        cur = [i]
        for k in range(1, len(ys) + 1):
            cost = 0 if xs[i - 1] == ys[k - 1] else 1
            cur.append(min(prev[k] + 1, cur[k - 1] + 1, prev[k - 1] + cost))
        prev = cur
    return prev[len(ys)]


def closest(word, dict_):
    best = (dict_[0], distance(word, dict_[0]))
    for cand in dict_:
        d = distance(word, cand)
        if d < best[1]:
            best = (cand, d)
    return best


def main():
    pairs = [
        ["kitten", "sitting"],
        ["flaw", "lawn"],
        ["saturday", "sunday"],
        ["same", "same"],
        ["", "abc"],
        ["abc", ""],
    ]
    for p in pairs:
        print(f"{p[0]} -> {p[1]}: {distance(p[0], p[1])}")
    dict_ = ["apple", "banana", "orange", "grape", "melon", "lemon", "peach"]
    for typo in ["aple", "bananna", "orrange", "grap", "lemmon", "pech", "kiwi"]:
        s = closest(typo, dict_)
        print(f"{typo}: did you mean {s[0]}? (distance {s[1]})")


main()
