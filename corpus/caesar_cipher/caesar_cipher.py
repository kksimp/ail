def rotate(c, base, k):
    return chr((ord(c) - base + k) % 26 + base)


def shift(text, key):
    k = (key % 26 + 26) % 26
    out = ""
    for c in text:
        if "a" <= c <= "z":
            ch = rotate(c, 97, k)
        elif "A" <= c <= "Z":
            ch = rotate(c, 65, k)
        else:
            ch = c
        out += ch
    return out


def score(text):
    n = 0
    for c in text.lower():
        if c in "etaoin":
            n += 1
    return n


def crack(cipher):
    best = 0
    best_score = -1
    for k in range(26):
        s = score(shift(cipher, -k))
        if s > best_score:
            best = k
            best_score = s
    return best


def main():
    plain = "Attack the east gate at noon, then retreat."
    enc = shift(plain, 3)
    print(enc)
    print(shift(enc, -3))
    print(shift(plain, 13))
    print("true" if shift(shift(plain, 13), 13) == plain else "false")
    secret = shift("the treasure is hidden in the old stone tower", 11)
    print(secret)
    k = crack(secret)
    print(f"guessed key {k}: {shift(secret, -k)}")


main()
