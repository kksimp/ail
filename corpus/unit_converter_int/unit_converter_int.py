UNITS = [
    ("mm", "length", 1000),
    ("cm", "length", 10000),
    ("m", "length", 1000000),
    ("km", "length", 1000000000),
    ("in", "length", 25400),
    ("ft", "length", 304800),
    ("yd", "length", 914400),
    ("mi", "length", 1609344000),
    ("mg", "mass", 1000),
    ("g", "mass", 1000000),
    ("kg", "mass", 1000000000),
    ("oz", "mass", 28349523),
    ("lb", "mass", 453592370),
    ("ml", "volume", 1000000),
    ("l", "volume", 1000000000),
    ("tsp", "volume", 4928922),
    ("cup", "volume", 236588236),
    ("gal", "volume", 3785411784),
]


class ConvError(Exception):
    pass


def find_unit(name):
    for u in UNITS:
        if u[0] == name:
            return u
    raise ConvError(f"unknown unit {name}")


def convert_hundredths(value, src, dst):
    return (value * src[2] * 200 + dst[2]) // (2 * dst[2])


def format_hundredths(q):
    return f"{q // 100}.{q % 100:02}"


def to_int(s):
    t = s[1:] if s.startswith("-") else s
    if t.isdigit() and t.isascii():
        return int(s)
    return None


def convert(query):
    parts = query.split()
    if len(parts) != 4 or parts[2] != "to":
        raise ConvError(f"cannot read query '{query}'")
    value = to_int(parts[0])
    if value is None:
        raise ConvError(f"bad number {parts[0]}")
    if value < 0 or value > 1000000:
        raise ConvError(f"bad number {parts[0]}")
    a = find_unit(parts[1])
    b = find_unit(parts[3])
    if a[1] != b[1]:
        raise ConvError(f"cannot convert {a[1]} to {b[1]}")
    return f"{value} {a[0]} = {format_hundredths(convert_hundredths(value, a, b))} {b[0]}"


def main():
    for q in [
        "5280 ft to mi",
        "1 mi to km",
        "100 cm to in",
        "3 lb to oz",
        "250 g to oz",
        "1 gal to cup",
        "2 l to tsp",
        "12 in to ft",
        "5 kg to mi",
        "7 parsec to m",
        "ten m to cm",
        "10 m cm",
        "0 m to km",
    ]:
        try:
            r = convert(q)
        except ConvError as err:
            print(f"{q} -> error: {err}")
            continue
        print(f"{q} -> {r}")


main()
