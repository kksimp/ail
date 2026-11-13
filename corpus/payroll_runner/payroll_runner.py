class PayError(Exception):
    pass


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


def parse_employee(text):
    parts = text.split(",")
    if len(parts) != 4:
        raise PayError(f"bad record '{text}'")
    name = parts[0]
    try:
        amount = int(parts[2])
        hours = int(parts[3])
    except ValueError:
        raise PayError(f"bad record '{text}'")
    if hours < 0 or hours > 80:
        raise PayError(f"{name}: hours out of range ({hours})")
    if parts[1] == "hourly":
        return (name, "hourly", amount, hours)
    if parts[1] == "salary":
        return (name, "salary", amount, 0)
    raise PayError(f"{name}: unknown pay kind {parts[1]}")


def gross_pay(e):
    name, kind, amount, hours = e
    if kind == "hourly":
        base = min(hours, 40)
        extra = hours - 40 if hours > 40 else 0
        return base * amount + extra * amount * 3 // 2
    return amount // 26


def income_tax(taxable):
    if taxable <= 100000:
        return taxable * 10 // 100
    return 10000 + (taxable - 100000) * 20 // 100


def slip_for(e):
    gross = gross_pay(e)
    pension = gross * 5 // 100
    tax = income_tax(gross - pension)
    return (e[0], gross, pension, tax, gross - pension - tax)


def main():
    records = [
        "ann,hourly,2250,45",
        "bob,salary,5200000,0",
        "cy,hourly,1800,38",
        "dee,hourly,2000,95",
        "eli,contract,100,10",
        "fay,hourly,abc,10",
        "gus,hourly,4000,40,1",
        "hal,hourly,3100,60",
    ]
    total_gross = 0
    total_net = 0
    print("name      gross   pension       tax       net")
    for rec in records:
        try:
            e = parse_employee(rec)
        except PayError as err:
            print(f"skipped: {err}")
            continue
        s = slip_for(e)
        total_gross += s[1]
        total_net += s[4]
        print(f"{s[0]:<5}{money(s[1]):>10}{money(s[2]):>10}{money(s[3]):>10}{money(s[4]):>10}")
    print(f"gross {money(total_gross)} net {money(total_net)}")


main()
