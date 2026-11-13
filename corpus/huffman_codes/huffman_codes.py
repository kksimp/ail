def lightest(weight, active):
    pick = 0
    for k in range(1, len(active)):
        a = active[k]
        b = active[pick]
        if weight[a] < weight[b] or (weight[a] == weight[b] and a < b):
            pick = k
    return pick


def build(text):
    freq = {}
    for c in text:
        freq[c] = freq.get(c, 0) + 1
    symbols = sorted(freq)
    weight, left, right, symbol = [], [], [], []
    active = []
    for s in symbols:
        active.append(len(weight))
        weight.append(freq[s])
        left.append(-1)
        right.append(-1)
        symbol.append(s)
    while len(active) > 1:
        i = lightest(weight, active)
        a = active[i]
        del active[i]
        j = lightest(weight, active)
        b = active[j]
        del active[j]
        active.append(len(weight))
        weight.append(weight[a] + weight[b])
        left.append(a)
        right.append(b)
        symbol.append("")
    return (weight, left, right, symbol, active[0])


def assign(tree, node, prefix, codes):
    weight, left, right, symbol, root = tree
    if left[node] == -1:
        codes[symbol[node]] = "0" if prefix == "" else prefix
        return
    assign(tree, left[node], prefix + "0", codes)
    assign(tree, right[node], prefix + "1", codes)


def encode(text, codes):
    return "".join(codes[c] for c in text)


def decode(bits, tree):
    weight, left, right, symbol, root = tree
    out = []
    cur = root
    for b in bits:
        if left[cur] != -1:
            cur = left[cur] if b == "0" else right[cur]
        if left[cur] == -1:
            out.append(symbol[cur])
            cur = root
    return "".join(out)


def show(label, text):
    tree = build(text)
    codes = {}
    assign(tree, tree[4], "", codes)
    names = sorted(codes)
    print(f"{label}: {len(text)} chars")
    for s in names:
        shown = "<sp>" if s == " " else s
        print(f"  {shown.ljust(4)} {codes[s]}")
    bits = encode(text, codes)
    back = decode(bits, tree)
    ok = "yes" if back == text else "no"
    print(f"  {len(bits)} bits vs {8 * len(text)}, round trip {ok}")


def main():
    show("sample", "abracadabra alakazam")
    show("skewed", "aaaaaaaabbbbccd")
    show("single", "zzzz")


main()
