"""Income tax return calculator in whole dollars."""


class TaxError(Exception):
    pass


DEDUCTION = {"single": 14000, "married": 28000, "head": 21000}


def brackets(status):
    k = 2 if status == "married" else 1
    return [(11000 * k, 10), (44000 * k, 12), (95000 * k, 22), (182000 * k, 24), (1000000000000, 35)]


def tax_on(taxable, status):
    prev = 0
    hundredths = 0
    for upto, rate in brackets(status):
        if taxable > prev:
            hundredths += (min(taxable, upto) - prev) * rate
        prev = upto
    return hundredths // 100


def prepare(line):
    p = line.split()
    if len(p) != 6:
        raise TaxError("cannot read return '%s'" % line)
    if p[1] not in DEDUCTION:
        raise TaxError("unknown filing status %s" % p[1])
    status = p[1]
    nums = []
    for text in p[2:6]:
        try:
            n = int(text)
        except ValueError:
            raise TaxError("cannot read return '%s'" % line)
        if n < 0:
            raise TaxError("%s reports a negative amount" % p[0])
        nums.append(n)
    gross = nums[0] + nums[1]
    taxable = max(gross - DEDUCTION[status], 0)
    tax = tax_on(taxable, status)
    return {"who": p[0], "gross": gross, "taxable": taxable, "tax": tax,
            "credit": min(nums[2] * 2000, tax), "withheld": nums[3]}


def describe(f):
    owed = f["tax"] - f["credit"]
    diff = owed - f["withheld"]
    eff = owed * 1000 // f["gross"] if f["gross"] > 0 else 0
    result = "refund %d" % -diff if diff <= 0 else "due %d" % diff
    return "%s: gross %d taxable %d tax %d credit %d owed %d -> %s (eff %d.%d%%)" % (
        f["who"], f["gross"], f["taxable"], f["tax"], f["credit"], owed, result, eff // 10, eff % 10)


def main():
    lines = [
        "ann single 52000 300 0 4800", "bo married 120000 2500 2 9000", "cy head 38000 0 1 2000",
        "di single 9000 100 0 0", "ed widow 50000 0 0 0", "flo single -500 0 0 0",
        "gus single abc 0 0 0", "hal married 400000 50000 3 70000", "ivy single 200000 0 0 30000",
    ]
    total = 0
    for line in lines:
        try:
            f = prepare(line)
        except TaxError as err:
            print("error: %s" % err)
            continue
        print(describe(f))
        total += f["tax"] - f["credit"]
    print("total tax owed %d" % total)


main()
