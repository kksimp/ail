def data():
    return [
        "id,service,price,quality",
        "r1,5,3,4",
        "r2,4,2,5",
        "r3,5,,4",
        "r4,3,3,3",
        "r5,2,1,2",
        "r6,4,4,9",
        "r7,5,2,5",
        "r8,,3,4",
        "r9,4,3,4",
        "r10,1,2,0",
    ]


def column(lines, col):
    out = []
    for line in lines:
        cells = line.split(",")
        try:
            v = int(cells[col])
        except ValueError:
            continue
        if v >= 1 and v <= 5:
            out.append(v)
    return out


def median(xs):
    s = sorted(xs)
    n = len(s)
    if n % 2 == 1:
        return s[n // 2]
    return (s[n // 2 - 1] + s[n // 2]) // 2


def hundredths(total, n):
    m = total * 100 // n
    rest = m % 100
    return f"{m // 100}.{rest:0>2}"


def favorable(xs):
    fav = 0
    for x in xs:
        if x >= 4:
            fav += 1
    return (fav * 100 + len(xs) // 2) // len(xs)


def counts(xs):
    out = [0, 0, 0, 0, 0]
    for x in xs:
        out[x - 1] += 1
    return out


def main():
    lines = data()[1:]
    names = ["service", "price", "quality"]
    print(f"responses: {len(lines)}")
    for q in range(len(names)):
        xs = column(lines, q + 1)
        cs = counts(xs)
        dist = " ".join(str(c) for c in cs)
        avg = hundredths(sum(xs), len(xs))
        med = median(xs)
        fav = favorable(xs)
        print(f"{names[q]:<8} n={len(xs)} counts[{dist}] mean {avg} median {med} favorable {fav}%")


main()
