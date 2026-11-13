def data():
    return [
        "sku,on_hand,daily,lead,pack",
        "bolt-m4,500,40,7,100",
        "nut-m4,120,30,5,50",
        "washer,900,25,10,200",
        "cable-1m,14,3,14,10",
        "gasket,0,2,9,6",
        "bad,line",
        "fuse-5a,61,6,6,25",
    ]


def parse(line):
    c = line.split(",")
    if len(c) != 5:
        return None
    try:
        return (c[0], int(c[1]), int(c[2]), int(c[3]), int(c[4]))
    except ValueError:
        return None


def reorder_point(s):
    return s[2] * s[3] + s[2] * 2


def packs_needed(s):
    gap = reorder_point(s) - s[1]
    if gap <= 0:
        return 0
    return (gap + s[4] - 1) // s[4]


def cover_days(s):
    return s[1] // s[2]


def main():
    items = []
    for line in data()[1:]:
        item = parse(line)
        if item is None:
            print(f"skipped: {line}")
            continue
        items.append(item)
    ordered = sorted(items, key=lambda x: cover_days(x))
    print("sku        stock cover  point order")
    total_packs = 0
    total_units = 0
    for s in ordered:
        rp = reorder_point(s)
        n = packs_needed(s)
        cover = cover_days(s)
        total_packs += n
        total_units += n * s[4]
        print(f"{s[0]:<10} {s[1]:>5} {cover:>5} {rp:>6} {n:>5}")
    print(f"order {total_packs} packs ({total_units} units)")


main()
