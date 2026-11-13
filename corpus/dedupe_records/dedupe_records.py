def data():
    return [
        "1, Ann Lee ,Ann.Lee@Example.com,20260101",
        "2,Bob Ray,bob@example.com,20260115",
        "3,Ann L.,ann.lee+news@example.com,20260220",
        "4,Cy Dunn,cy@example.org,20260301",
        "5,Bobby Ray,BOB@example.com ,20260110",
        "6,Dee,dee@example.org",
        "7,Cy D.,cy@example.org,20260301",
        "8,Eve Fox,eve@example.net,20260105",
    ]


def normalize(email):
    lower = email.strip().lower()
    parts = lower.split("@")
    if len(parts) != 2:
        return lower
    local = parts[0]
    p = local.find("+")
    if p >= 0:
        local = local[:p]
    return f"{local}@{parts[1]}"


def parse(line):
    cells = line.split(",")
    if len(cells) != 4:
        return None
    try:
        return (int(cells[0].strip()), cells[1].strip(), cells[2], int(cells[3]))
    except ValueError:
        return None


def dedupe(rows):
    kept = []
    pos = {}
    for c in rows:
        key = normalize(c[2])
        if key in pos:
            at = pos[key]
            if c[3] > kept[at][3]:
                kept[at] = c
        else:
            pos[key] = len(kept)
            kept.append(c)
    return kept


def main():
    rows = []
    for line in data():
        c = parse(line)
        if c is None:
            print(f"unparsed: {line}")
            continue
        rows.append(c)
    kept = sorted(dedupe(rows), key=lambda c: c[0])
    print(f"{len(rows)} records, {len(kept)} unique, {len(rows) - len(kept)} dropped")
    for k in kept:
        print(f"{k[0]:>2} {k[1]:<9} {normalize(k[2]):<22} {k[3]}")


main()
