def parse_stage(lines):
    out = []
    for line in lines:
        f = line.split(",")
        if len(f) != 4:
            continue
        try:
            qty = int(f[2])
            price = int(f[3])
        except ValueError:
            continue
        out.append((f[0], f[1], qty, price))
    return out


def main():
    lines = [
        "ann,pen,3,2",
        "bob,ink,x,5",
        "ann,pad,2,7",
        "cy,pen,0,2",
        "bob,pen,10,2",
        "dan,cap,1",
        "cy,ink,4,5",
        "",
        "ann,ink,1,5",
        "bob,pad,5,7",
    ]
    parsed = parse_stage(lines)
    print(f"{'parse':<10} in {len(lines):>2} out {len(parsed):>2}")
    valid = [o for o in parsed if o[2] > 0]
    print(f"{'validate':<10} in {len(parsed):>2} out {len(valid):>2}")
    amounts = [o[2] * o[3] for o in valid]
    print(f"{'price':<10} in {len(valid):>2} out {len(amounts):>2}")
    by = {}
    for o, a in zip(valid, amounts):
        n, amt = by.get(o[0], (0, 0))
        by[o[0]] = (n + 1, amt + a)
    print(f"{'aggregate':<10} in {len(amounts):>2} out {len(by):>2}")
    for c in sorted(by):
        print(f"{c:<4}{by[c][0]:>2} orders {by[c][1]:>4}")
    print(f"grand total {sum(amounts)}")


main()
