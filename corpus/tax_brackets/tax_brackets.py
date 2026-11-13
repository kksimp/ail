TOP = 100000000000000


class TaxError(Exception):
    pass


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


def validate(brackets):
    for i in range(1, len(brackets)):
        if brackets[i][0] <= brackets[i - 1][0]:
            raise TaxError(f"bracket {i} does not rise above the one before")


def slices(brackets, income):
    validate(brackets)
    if income < 0:
        raise TaxError(f"income cannot be negative: {income}")
    prev = 0
    out = []
    for upto, pct in brackets:
        top = min(income, upto)
        out.append(top - prev if top > prev else 0)
        prev = upto
    return out


def tax_for(brackets, income):
    parts = slices(brackets, income)
    tax = 0
    for i in range(len(parts)):
        tax += parts[i] * brackets[i][1] // 100
    return tax


def marginal(brackets, income):
    for upto, pct in brackets:
        if income < upto:
            return pct
    return brackets[-1][1] if brackets else 0


def tenths(per_mille):
    return f"{per_mille // 10}.{per_mille % 10}%"


def report(name, brackets, income):
    try:
        tax = tax_for(brackets, income)
    except TaxError as err:
        print(f"{name} {income}: {err}")
        return
    eff = tax * 1000 // income if income > 0 else 0
    print(f"{name} {money(income)}: tax {money(tax)}, effective {tenths(eff)}, marginal {marginal(brackets, income)}%")


def main():
    single = [(1100000, 10), (4472500, 12), (9525000, 22), (TOP, 32)]
    flat = [(5000000, 0), (TOP, 15)]
    broken = [(500, 5), (400, 9)]
    for income in [0, 850000, 4000000, 12000000, -100]:
        report("single", single, income)
        report("flat", flat, income)
    report("broken", broken, 1000)
    parts = slices(single, 6000000)
    for i in range(len(parts)):
        print(f"slice {i + 1}: {money(parts[i])} at {single[i][1]}%")


main()
