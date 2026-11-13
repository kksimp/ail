DATA = [
    "name,age,city,active",
    "ada,36,London,true",
    "bob,41,New York,false",
    "bad,row",
    'Dee "D" Lee,25,Rome,false',
    "linus,,Helsinki,true",
    "grace,-3,Paris,true",
]


def quote(s):
    esc = s.replace("\\", "\\\\").replace('"', '\\"')
    return '"' + esc + '"'


def value(cell):
    if cell == "":
        return "null"
    if cell in ("true", "false"):
        return cell
    if cell.lstrip("-").isdigit() and cell.count("-") <= 1 and not cell.endswith("-"):
        return str(int(cell))
    return quote(cell)


def obj(headers, cells):
    parts = [f"{quote(headers[i])}: {value(cells[i])}" for i in range(len(headers))]
    return "{" + ", ".join(parts) + "}"


def convert(lines, skipped):
    headers = [h.strip() for h in lines[0].split(",")]
    objs = []
    for i in range(1, len(lines)):
        cells = [c.strip() for c in lines[i].split(",")]
        if len(cells) != len(headers):
            skipped.append(f"line {i + 1}: expected {len(headers)} cells, got {len(cells)}")
            continue
        objs.append(obj(headers, cells))
    return objs


def main():
    skipped = []
    objs = convert(DATA, skipped)
    print("[")
    for i in range(len(objs)):
        comma = "," if i < len(objs) - 1 else ""
        print(f"  {objs[i]}{comma}")
    print("]")
    print(f"{len(objs)} objects, {len(skipped)} skipped")
    for s in skipped:
        print(f"skipped {s}")


main()
