def customers():
    return ["1,Ada,Paris", "2,Bob,Oslo", "3,Cy,Rome", "4,Dee,Oslo", "5,Eve,Lima"]


def orders():
    return [
        "100,1,250",
        "101,3,90",
        "102,1,40",
        "103,9,500",
        "104,4,15",
        "105,3,x",
        "106,1,310",
        "107,7,60",
    ]


def load_names():
    names = {}
    for line in customers():
        cells = line.split(",")
        names[int(cells[0])] = f"{cells[1]} ({cells[2]})"
    return names


def load_orders():
    out = []
    for line in orders():
        cells = line.split(",")
        try:
            out.append((int(cells[0]), int(cells[1]), int(cells[2])))
        except ValueError:
            continue
    return out


def main():
    names = load_names()
    all_orders = load_orders()
    print("inner join:")
    totals = {}
    orphans = []
    for o in all_orders:
        if o[1] not in names:
            orphans.append(o)
            continue
        who = names[o[1]]
        totals[o[1]] = totals.get(o[1], 0) + o[2]
        print(f"  order {o[0]}: {who} {o[2]}")
    print("customers without orders:")
    for cid in sorted(names):
        if cid not in totals:
            print(f"  {names[cid]}")
    print("orphan orders:")
    for o in orphans:
        print(f"  order {o[0]} refers to unknown customer {o[1]}")
    print("totals:")
    ranked = sorted(sorted(totals), key=lambda k: -totals[k])
    for cid in ranked:
        t = totals[cid]
        print(f"  {names[cid]:<14}{t:>5}")


main()
