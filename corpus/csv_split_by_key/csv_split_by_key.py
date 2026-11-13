import os, tempfile


def sample():
    rows = [
        "region,item,qty",
        "north,bolt,12",
        "south,nut,7",
        "north,washer,30",
        "east,bolt,5",
        "south,bolt,9",
        "north,nut,4",
        "",
        "east,screw,x",
    ]
    return "\n".join(rows)


def group_rows(text, col):
    groups = {}
    first = True
    for line in text.splitlines():
        if first:
            first = False
            continue
        if line.strip() == "":
            continue
        cells = line.split(",")
        if len(cells) <= col:
            continue
        groups.setdefault(cells[col], []).append(line)
    return groups


def safe_name(key):
    return f"part_{key.lower()}.csv"


def write_parts(d, header, groups):
    names = []
    for key in sorted(groups):
        name = safe_name(key)
        body = header + "\n" + "\n".join(groups[key]) + "\n"
        with open(os.path.join(d, name), "w", newline="") as f:
            f.write(body)
        names.append(name)
    return names


def total_qty(text):
    s = 0
    first = True
    for line in text.splitlines():
        if first:
            first = False
            continue
        cells = line.split(",")
        try:
            s += int(cells[-1])
        except ValueError:
            pass
    return s


def main():
    d = tempfile.mkdtemp(prefix="corpus_csv_split_by_key")
    text = sample()
    header = text.splitlines()[0]
    groups = group_rows(text, 0)
    names = write_parts(d, header, groups)
    print(f"wrote {len(names)} files for key 'region'")
    grand = 0
    for name in names:
        with open(os.path.join(d, name), newline="") as f:
            body = f.read()
        rows = len(body.splitlines()) - 1
        sub = total_qty(body)
        grand += sub
        print(f"{name:<16}{rows:>2} rows  qty {sub:>3}")
    print(f"grand total {grand}, original {total_qty(text)}")
    for name in names:
        os.remove(os.path.join(d, name))
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
