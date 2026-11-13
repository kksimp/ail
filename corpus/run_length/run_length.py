def encode(text):
    out = ""
    prev = " "
    count = 0
    for c in text:
        if count > 0 and c == prev:
            count += 1
        else:
            if count > 0:
                out += "%d%s" % (count, prev)
            prev = c
            count = 1
    if count > 0:
        out += "%d%s" % (count, prev)
    return out


def decode(coded):
    out = ""
    num = ""
    for c in coded:
        if c.isdigit():
            num += c
            continue
        if num == "":
            return None
        out += c * int(num)
        num = ""
    if num != "":
        return None
    return out


def percent(small, big):
    if big == 0:
        return 100
    return 100 * small // big


def report(text):
    coded = encode(text)
    back = decode(coded)
    if back is None:
        back = "?"
    ok = back == text
    pct = percent(len(coded), len(text))
    print("%-22s %-18s %4d%%  round trip: %s" % (text, coded, pct, "true" if ok else "false"))


def main():
    for s in [
        "aaaabbbcca",
        "WWWWWWWWWWWWBWWWWWWWWWWWWBBBWWWWWWWWWWWWWWWWWWWWWWWWBWWWWWWWWWWWWWW",
        "abcdef",
        "zzzzzzzzzzzzzzzzzzzz",
        "x",
        "",
    ]:
        report(s)
    bad = ["3a2", "ab", "a3", "12"]
    for b in bad:
        r = decode(b)
        if r is None:
            print("decode %s: invalid" % b)
        else:
            print("decode %s: ok" % b)


main()
