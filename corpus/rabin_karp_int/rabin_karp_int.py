BASE = 256


def power_mod(base, exp, modulus):
    result = 1
    for _ in range(exp):
        result = result * base % modulus
    return result


def hash_of(codes, start, size, modulus):
    h = 0
    for k in range(size):
        h = (h * BASE + codes[start + k]) % modulus
    return h


def search(text, pattern, modulus):
    t = list(text)
    p = list(pattern)
    tcodes = [ord(c) for c in t]
    pcodes = [ord(c) for c in p]
    m = len(p)
    hits = []
    spurious = 0
    if m == 0 or m > len(t):
        return hits, 0
    top = power_mod(BASE, m - 1, modulus)
    want = hash_of(pcodes, 0, m, modulus)
    h = hash_of(tcodes, 0, m, modulus)
    for i in range(len(t) - m + 1):
        if h == want:
            if t[i:i + m] == p:
                hits.append(i)
            else:
                spurious += 1
        if i + m < len(t):
            drop = tcodes[i] * top % modulus
            h = ((h + modulus - drop) * BASE + tcodes[i + m]) % modulus
    return hits, spurious


def main():
    text = "the cat sat on the mat with the other cat near the threshold"
    print(f"text: {text}")
    for pat in ["the", "cat", "at ", "thresh", "dog"]:
        small = search(text, pat, 101)
        large = search(text, pat, 1000003)
        print(f"{pat}: hits [{' '.join(str(x) for x in small[0])}], spurious mod 101: {small[1]}, mod 1000003: {large[1]}")
    dna = "ACGTACGTTAGCACGTACGGACGT"
    found = search(dna, "ACGT", 1000003)
    print(f"dna: {len(found[0])} matches at [{' '.join(str(x) for x in found[0])}]")


main()
