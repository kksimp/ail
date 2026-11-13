def load_dict(entries):
    d = {}
    for e in entries:
        parts = e.split(" ")
        d[parts[0]] = int(parts[1])
    return d


def edits1(word):
    letters = "abcdefghijklmnopqrstuvwxyz"
    n = len(word)
    out = set()
    for i in range(n + 1):
        left = word[0:i]
        right = word[i:n]
        if len(right) > 0:
            out.add(left + right[1:])
        if len(right) > 1:
            out.add(left + right[1:2] + right[0:1] + right[2:])
        for c in letters:
            if len(right) > 0:
                out.add(left + c + right[1:])
            out.add(left + c + right)
    return out


def suggest(word, d):
    if word in d:
        return [word]
    found = []
    for cand in sorted(edits1(word)):
        if cand in d:
            found.append(cand)
    return sorted(found, key=lambda w: -d[w])


def main():
    d = load_dict(["the 500", "tea 40", "ten 35", "then 90", "spelling 12", "spoiling 3", "spewing 2", "correct 60", "corrects 4", "collect 30", "word 80", "ward 20", "world 70", "would 85", "cat 25", "cart 15", "act 18", "hello 55", "help 45"])
    for w in ["teh", "speling", "korrect", "wrld", "wuold", "cta", "hello", "zzzz", "hel"]:
        got = suggest(w, d)
        if not got:
            print("%s -> (none)" % w)
        else:
            print("%s -> %s" % (w, ", ".join(got)))
    print("candidates for speling: %d" % len(edits1("speling")))


main()
