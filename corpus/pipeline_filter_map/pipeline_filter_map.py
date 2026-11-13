import re


def parse_record(line):
    parts = line.split(",")
    if len(parts) != 3:
        return None
    a, b = parts[0].strip(), parts[1].strip()
    if not re.fullmatch(r"-?\d+", a) or not re.fullmatch(r"-?\d+", b):
        return None
    i, c = int(a), int(b)
    if i < 0 or c < 0:
        return None
    return (i, c, parts[2].strip())


def dollars(cents):
    return f"{cents // 100}.{cents % 100:02}"


def main():
    lines = [
        "101,2500,ok",
        "102,1999,refunded",
        "not a record",
        "103,0,ok",
        "104,10000,ok",
        "105,abc,ok",
        "106,-5,ok",
        "107,799,ok",
        "108,450,pending",
        "109,1250, ok",
    ]
    recs = [r for r in (parse_record(l) for l in lines) if r is not None]
    out = [(i, c + c * 8 // 100, t) for i, c, t in recs if t == "ok" and c > 0]
    print(f"{len(lines)} lines, {len(recs)} parsed, {len(out)} kept")
    total = 0
    for i, c, t in out:
        print(f"order {i}: {dollars(c):>8}")
        total += c
    print(f"total with tax: {dollars(total)}")


main()
