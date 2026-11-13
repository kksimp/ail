"""Recipe scaler in integer arithmetic."""


class ScaleError(Exception):
    pass


UNITS = {"g", "ml", "tbsp", "tsp", "pc"}


def parse_line(line):
    p = line.split()
    if len(p) < 3:
        raise ScaleError("cannot read '%s'" % line)
    try:
        n = int(p[0])
    except ValueError:
        raise ScaleError("cannot read '%s'" % line)
    if p[1] not in UNITS:
        raise ScaleError("unknown unit %s" % p[1])
    return (n, p[1], p[2])


def scale_qty(q, frm, to):
    scaled = (q * to * 2 + frm) // (2 * frm)
    if q > 0 and scaled == 0:
        return 1
    return scaled


def thousandths(q, big):
    return "%d.%03d %s" % (q // 1000, q % 1000, big)


def show(q, u):
    if u == "g":
        return thousandths(q, "kg") if q >= 1000 else "%d g" % q
    if u == "ml":
        return thousandths(q, "l") if q >= 1000 else "%d ml" % q
    if u == "tbsp":
        return "%d tbsp" % q
    if u == "tsp":
        return "%d tbsp" % (q // 3) if q >= 3 and q % 3 == 0 else "%d tsp" % q
    return "%d pc" % q


def scale_recipe(lines, frm, to):
    if to < 1 or frm < 1:
        raise ScaleError("cannot scale to %d servings" % to)
    out = []
    for line in lines:
        qty, unit, name = parse_line(line)
        out.append(name.ljust(8) + show(scale_qty(qty, frm, to), unit))
    return out


def main():
    base = ["200 g flour", "300 ml milk", "2 pc egg", "1 tsp salt", "2 tbsp sugar", "40 g butter"]
    for servings in [4, 6, 1, 20, 0]:
        print("serves %d:" % servings)
        try:
            rows = scale_recipe(base, 4, servings)
        except ScaleError as err:
            print("  error: %s" % err)
            continue
        for row in rows:
            print("  " + row)
    for bad in [["5 cups rice"], ["x g salt"], ["3 g"]]:
        try:
            rows = scale_recipe(bad, 1, 2)
        except ScaleError as err:
            print("error: %s" % err)
            continue
        print(len(rows))


main()
