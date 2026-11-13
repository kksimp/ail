def merge_text(m, text):
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines = lines[:-1]
    header = lines[0].split(",")
    if "id" not in header:
        return 0
    id_pos = header.index("id")
    for h in header:
        if h not in m["cols"]:
            m["cols"].append(h)
    skipped = 0
    for line in lines[1:]:
        parts = line.split(",")
        if len(parts) != len(header):
            skipped += 1
            continue
        try:
            i_d = int(parts[id_pos])
        except ValueError:
            skipped += 1
            continue
        if i_d not in m["ids"]:
            m["ids"].append(i_d)
        for i in range(len(header)):
            if parts[i] != "":
                m["cells"][f"{i_d}:{header[i]}"] = parts[i]
    return skipped


def render(m):
    out = [",".join(m["cols"])]
    for i_d in sorted(m["ids"]):
        out.append(",".join(m["cells"].get(f"{i_d}:{c}", "") for c in m["cols"]))
    return out


def main():
    names = ["people.csv", "scores.csv", "fixes.csv"]
    texts = [
        "id,name,city\n3,Cy,Oslo\n1,Ann,Rome\n2,Bob,\n",
        "score,id\n88,1\n75,2\n91,4\nbad row\n60,x\n",
        "id,city,name\n2,Lima,Robert\n4,Kyiv,Dee\n",
    ]
    m = {"cols": [], "ids": [], "cells": {}}
    skipped = 0
    for i in range(len(names)):
        skipped += merge_text(m, texts[i])
        print(f"after {names[i]}: {len(m['ids'])} ids, columns {'/'.join(m['cols'])}")
    rows = render(m)
    print(f"skipped {skipped} bad rows")
    for line in rows:
        print(f"  {line}")
    empty = 0
    for r in rows[1:]:
        empty += sum(1 for cell in r.split(",") if cell == "")
    print(f"rows {len(rows) - 1}, empty cells {empty}")
    print("cleaned up: true")


main()
