def is_number(s):
    if s == "":
        return False
    for c in s:
        if c not in "0123456789":
            return False
    return True


def column(rows, j):
    return [r[j] if j < len(r) else "" for r in rows]


def stats_for(name, cells):
    seen = set()
    empty = 0
    numeric = True
    filled = []
    for c in cells:
        if c == "":
            empty += 1
            continue
        filled.append(c)
        seen.add(c)
        if not is_number(c):
            numeric = False
    if not filled:
        return dict(name=name, numeric=False, filled=0, empty=empty, distinct=0, low="-", high="-", sum=0)
    if numeric:
        nums = [int(c) for c in filled]
        return dict(name=name, numeric=True, filled=len(nums), empty=empty, distinct=len(seen),
                    low=str(min(nums)), high=str(max(nums)), sum=sum(nums))
    st = sorted(filled)
    return dict(name=name, numeric=False, filled=len(filled), empty=empty, distinct=len(seen),
                low=st[0], high=st[-1], sum=0)


def analyse(text):
    lines = text.split("\n")
    header = lines[0].split("\t")
    rows = []
    for line in lines[1:]:
        if line != "":
            rows.append(line.split("\t"))
    return [stats_for(header[j], column(rows, j)) for j in range(len(header))]


def data():
    rows = [
        "city\tpop\tcountry\tarea",
        "Oslo\t709\tNO\t454",
        "Bergen\t286\tNO\t465",
        "Malmo\t350\tSE\t",
        "Lund\t95\tSE\t30",
        "Aarhus\t285\tDK\t91",
        "Tromso\t77\tNO",
        "Oslo\t709\tNO\t454",
    ]
    return "\n".join(rows)


for s in analyse(data()):
    kind = "num" if s["numeric"] else "text"
    print(f"{s['name']:<8}{kind:<5}n={s['filled']} empty={s['empty']} distinct={s['distinct']} min={s['low']} max={s['high']}")
    if s["numeric"]:
        print(f"        sum={s['sum']} mean={s['sum'] // s['filled']}")
