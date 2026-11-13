def data():
    return [
        "region,product,units,price",
        "north,widget,10,25",
        "south,gadget,4,80",
        "north,gadget,3,80",
        "east,widget,7,25",
        "south,widget,12,25",
        "west,gizmo,2,150",
        "north,widget,5,25",
        "broken,row",
        "east,gizmo,1,150",
        "south,gadget,x,80",
        "east,gadget,6,80",
    ]


def parse(line):
    cells = line.split(",")
    if len(cells) != 4:
        return None
    try:
        return (cells[0], cells[1], int(cells[2]), int(cells[3]))
    except ValueError:
        return None


def main():
    sales = []
    bad = 0
    for line in data()[1:]:
        s = parse(line)
        if s is None:
            bad += 1
            continue
        sales.append(s)
    regions = sorted(set(s[0] for s in sales))
    products = sorted(set(s[1] for s in sales))
    cells = {}
    for s in sales:
        k = s[0] + "|" + s[1]
        cells[k] = cells.get(k, 0) + s[2] * s[3]
    print(f"records: {len(sales)}, skipped: {bad}")
    head = "region  "
    for p in products:
        head = f"{head}{p:>8}"
    label = "total"
    print(f"{head}{label:>8}")
    col_totals = {}
    grand = 0
    for r in regions:
        line = f"{r:<8}"
        row_total = 0
        for p in products:
            v = cells.get(r + "|" + p, 0)
            line = f"{line}{v:>8}"
            row_total += v
            col_totals[p] = col_totals.get(p, 0) + v
        grand += row_total
        print(f"{line}{row_total:>8}")
    foot = "total   "
    for p in products:
        t = col_totals[p]
        foot = f"{foot}{t:>8}"
    print(f"{foot}{grand:>8}")


main()
