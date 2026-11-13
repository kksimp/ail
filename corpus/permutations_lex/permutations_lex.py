def factorial(n):
    f = 1
    for k in range(2, n + 1):
        f *= k
    return f


def next_perm(xs):
    i = len(xs) - 2
    while i >= 0 and xs[i] >= xs[i + 1]:
        i -= 1
    if i < 0:
        xs.reverse()
        return False
    j = len(xs) - 1
    while xs[j] <= xs[i]:
        j -= 1
    xs[i], xs[j] = xs[j], xs[i]
    tail = xs[i + 1:][::-1]
    for k in range(len(tail)):
        xs[i + 1 + k] = tail[k]
    return True


def rank_of(xs):
    n = len(xs)
    rank = 0
    for i in range(n):
        smaller = 0
        for j in range(i + 1, n):
            if xs[j] < xs[i]:
                smaller += 1
        rank += smaller * factorial(n - 1 - i)
    return rank


def unrank(n, index):
    pool = list(range(n))
    left = index
    out = []
    for i in range(n):
        f = factorial(n - 1 - i)
        pick = left // f
        left = left % f
        out.append(pool[pick])
        del pool[pick]
    return out


def spell(xs, letters):
    return "".join(letters[x] for x in xs)


def all_arrangements(word):
    letters = sorted(word)
    names = []
    codes = []
    for c in letters:
        if c in names:
            codes.append(names.index(c))
        else:
            names.append(c)
            codes.append(len(names) - 1)
    out = [spell(codes, names)]
    while next_perm(codes):
        out.append(spell(codes, names))
    return out


def j(xs):
    return "".join(str(x) for x in xs)


def main():
    base = [1, 2, 3]
    cur = list(base)
    line = [j(cur)]
    while next_perm(cur):
        line.append(j(cur))
    print("perms of 123: " + " ".join(line))
    print("arrangements of aab: " + " ".join(all_arrangements("aab")))
    words = all_arrangements("banana")
    print(f"banana: {len(words)} distinct, first {words[0]}, last {words[-1]}")
    for k in [0, 1, 23, 100, 719]:
        p = unrank(6, k)
        print(f"unrank(6, {k}) = {j(p)} rank {rank_of(p)}")
    print(f"rank of 20413: {rank_of([2, 0, 4, 1, 3])}")


main()
