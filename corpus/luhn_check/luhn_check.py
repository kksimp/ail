def digits_of(s):
    ds = []
    for c in s:
        if c == " " or c == "-":
            continue
        if not ("0" <= c <= "9"):
            return None
        ds.append(int(c))
    return ds


def luhn_sum(ds):
    total = 0
    for i in range(len(ds)):
        v = ds[len(ds) - 1 - i]
        if i % 2 == 1:
            v = v * 2
            if v > 9:
                v -= 9
        total += v
    return total


def is_valid(s):
    ds = digits_of(s)
    if ds is None:
        return False
    return len(ds) >= 2 and luhn_sum(ds) % 10 == 0


def check_digit(payload):
    ds = list(payload)
    ds.append(0)
    return (10 - luhn_sum(ds) % 10) % 10


def with_check(payload):
    ds = digits_of(payload)
    if ds is None:
        return ""
    return "%s%d" % (payload, check_digit(ds))


def issuer(s):
    ds = digits_of(s)
    if ds is None:
        return "unknown"
    if len(ds) < 2:
        return "unknown"
    two = ds[0] * 10 + ds[1]
    if ds[0] == 4:
        return "visa"
    if 51 <= two <= 55:
        return "mastercard"
    if two == 34 or two == 37:
        return "amex"
    if two == 60 or two == 65:
        return "discover"
    return "other"


def main():
    samples = ["79927398713", "4111 1111 1111 1111", "5500-0000-0000-0004", "3782 822463 10005", "6011111111111117", "4111 1111 1111 1112", "1234 5678 9012 3456", "12a4", "0", "8"]
    for s in samples:
        verdict = "valid" if is_valid(s) else "invalid"
        print("%-22s %-8s %s" % (s, verdict, issuer(s)))
    for payload in ["7992739871", "411111111111111", "37828224631000"]:
        full = with_check(payload)
        print("%s -> %s (%s)" % (payload, full, "true" if is_valid(full) else "false"))


main()
