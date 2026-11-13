class LoanError(Exception):
    pass


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


def interest_of(balance, bps):
    return (balance * bps + 60000) // 120000


def clears(principal, bps, months, pay):
    bal = principal
    for step in range(months):
        due = bal + interest_of(bal, bps)
        if due <= pay:
            bal = 0
        else:
            bal = due - pay
    return bal == 0


def level_payment(principal, bps, months):
    lo = 0
    hi = principal * 2
    while lo < hi:
        mid = (lo + hi) // 2
        if clears(principal, bps, months, mid):
            hi = mid
        else:
            lo = mid + 1
    return lo


def schedule(principal, bps, months):
    if principal <= 0:
        raise LoanError(f"principal must be positive, got {principal}")
    if bps < 0 or bps > 100000:
        raise LoanError(f"rate out of range: {bps}")
    if months < 1 or months > 480:
        raise LoanError(f"term must be 1-480 months, got {months}")
    pay = level_payment(principal, bps, months)
    bal = principal
    rows = []
    total = 0
    for m in range(1, months + 1):
        i = interest_of(bal, bps)
        due = bal + i
        paid = due if due < pay else pay
        bal = due - paid
        total += i
        rows.append((m, paid, i, bal))
    return (pay, rows, total)


def show(principal, bps, months):
    try:
        plan = schedule(principal, bps, months)
    except LoanError as err:
        print(f"loan {principal} cents at {bps}bps for {months}: {err}")
        return
    print(f"loan {principal} cents at {bps}bps for {months} months: payment {money(plan[0])}, interest {money(plan[2])}")
    last = len(plan[1]) - 1
    for k in [0, 1, last]:
        r = plan[1][k]
        print(f"  month {r[0]:>3}: paid {money(r[1]):>9} interest {money(r[2]):>8} balance {money(r[3]):>10}")


show(1000000, 600, 12)
show(25000000, 450, 360)
show(120000, 0, 10)
show(-5, 500, 12)
show(50000, 100001, 12)
show(50000, 500, 0)
