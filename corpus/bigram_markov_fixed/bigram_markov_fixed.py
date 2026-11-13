TEXT = "the cat sat on the mat and the dog sat on the rug while the cat watched the dog and the dog watched the cat on the mat"


def learn(follows, text):
    ws = text.split()
    for i in range(1, len(ws)):
        follows.setdefault(ws[i - 1], []).append(ws[i])


def branching(follows, w):
    opts = follows.get(w)
    if opts is None:
        return 0
    return len(set(opts))


def next_rand(s):
    return (s * 1103515245 + 12345) % 2147483648


def generate(follows, start, seed, max_words):
    cur = start
    s = seed
    out = [start]
    for i in range(1, max_words):
        opts = follows.get(cur)
        if opts is None:
            break
        s = next_rand(s)
        cur = opts[s // 65536 % len(opts)]
        out.append(cur)
    return " ".join(out)


def main():
    follows = {}
    learn(follows, TEXT)
    names = sorted(follows)
    print(f"{len(names)} distinct words with followers")
    for w in names:
        n = branching(follows, w)
        if n >= 2:
            print(f"{w}: {n} different followers, {len(follows[w])} total")
    print(generate(follows, "the", 1, 8))
    print(generate(follows, "dog", 2, 8))
    print(generate(follows, "cat", 3, 8))


main()
