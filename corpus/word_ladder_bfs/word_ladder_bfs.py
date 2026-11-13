def one_apart(a, b):
    if len(a) != len(b):
        return False
    return sum(1 for x, y in zip(a, b) if x != y) == 1


def climb(words, start, goal):
    if start not in words or goal not in words:
        return ([], 0)
    src, dst = words.index(start), words.index(goal)
    prev = [-2] * len(words)
    prev[src] = -1
    queue = [src]
    head = 0
    while head < len(queue) and prev[dst] == -2:
        cur = queue[head]
        head += 1
        for i in range(len(words)):
            if prev[i] == -2 and one_apart(words[cur], words[i]):
                prev[i] = cur
                queue.append(i)
    chain = []
    if prev[dst] != -2:
        at = dst
        while at != -1:
            chain.append(words[at])
            at = prev[at]
        chain.reverse()
    return (chain, len(queue))


def report(words, start, goal):
    path, visited = climb(words, start, goal)
    if not path:
        print(f"{start} -> {goal}: no ladder ({visited} words visited)")
    else:
        print(f"{start} -> {goal}: {len(path) - 1} steps ({visited} words visited)")
        print("  " + " > ".join(path))


def main():
    words = [
        "cold", "cord", "card", "ward", "warm", "core", "care", "cure", "pure",
        "pore", "pole", "pale", "male", "mole", "hole", "hold", "bold", "bolt",
        "boat", "coat", "cost", "most", "mist", "list", "lost", "lose", "love",
        "dove", "wolf", "gold",
    ]
    report(words, "cold", "warm")
    report(words, "care", "love")
    report(words, "boat", "mist")
    report(words, "cold", "wolf")
    report(words, "cold", "zzzz")


def tests():
    assert one_apart("cold", "cord") and not one_apart("cold", "cold")
    assert not one_apart("cold", "card") and not one_apart("cold", "colds")
    p, _ = climb(["cat", "cot", "cog", "dog", "dot"], "cat", "dog")
    assert len(p) == 4 and p[0] == "cat" and p[3] == "dog"
    assert climb(["aaa", "bbb"], "aaa", "bbb")[0] == []


tests()
main()
