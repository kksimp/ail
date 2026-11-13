class InvoiceError(Exception):
    pass


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


def parse_line(text):
    parts = text.split(",")
    if len(parts) != 4:
        raise InvoiceError(f"bad line '{text}'")
    try:
        qty = int(parts[1])
        price = int(parts[2])
        bps = int(parts[3])
    except ValueError:
        raise InvoiceError(f"bad line '{text}'")
    if qty <= 0:
        raise InvoiceError(f"bad quantity {qty} for {parts[0]}")
    return (parts[0], qty, price, bps)


def discount_pct(subtotal):
    if subtotal >= 50000:
        return 10
    if subtotal >= 10000:
        return 5
    return 0


def line_tax(l):
    return (l[1] * l[2] * l[3] + 5000) // 10000


def compute(lines):
    gross = 0
    tax = 0
    for l in lines:
        gross += l[1] * l[2]
        tax += line_tax(l)
    disc = gross * discount_pct(gross) // 100
    return (gross, disc, tax, gross - disc + tax)


def main():
    invoices = [
        ["A100,2,1999,825", "B200,10,500,0", "C300,1,12000,825"],
        ["D1,0,100,0", "E2,3,250,500", "bad line"],
        ["F9,40,1500,825", "G7,x,100,0"],
    ]
    n = 0
    for inv in invoices:
        n += 1
        lines = []
        for text in inv:
            try:
                ln = parse_line(text)
            except InvoiceError as err:
                print(f"invoice {n}: skip: {err}")
                continue
            lines.append(ln)
        t = compute(lines)
        print(f"invoice {n}: {len(lines)} lines, subtotal {money(t[0])}, discount {money(t[1])}, tax {money(t[2])}, total {money(t[3])}")


main()
