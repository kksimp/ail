class LedgerError(Exception):
    pass


class Ledger:
    def __init__(self):
        self.balances = {}
        self.applied = 0

    def debit(self, who, amt):
        if who not in self.balances:
            raise LedgerError(f"no such account {who}")
        bal = self.balances[who]
        if bal < amt:
            raise LedgerError(f"{who} is short by {amt - bal}")
        self.balances[who] = bal - amt

    def apply(self, line):
        parts = line.split(" ")
        try:
            amt = int(parts[-1])
        except ValueError:
            raise LedgerError(f"cannot parse '{line}'")
        cmd = parts[0]
        if cmd == "deposit" and len(parts) == 3:
            self.balances[parts[1]] = self.balances.get(parts[1], 0) + amt
        elif cmd == "withdraw" and len(parts) == 3:
            self.debit(parts[1], amt)
        elif cmd == "transfer" and len(parts) == 4:
            self.debit(parts[1], amt)
            self.balances[parts[2]] = self.balances.get(parts[2], 0) + amt
        else:
            raise LedgerError(f"cannot parse '{line}'")
        self.applied += 1
        return amt


def money(cents):
    rest = cents % 100
    return f"{cents // 100}.{rest:0>2}"


def main():
    lines = [
        "deposit alice 10000",
        "deposit bob 2550",
        "withdraw alice 2599",
        "transfer alice bob 5000",
        "withdraw bob 9999",
        "transfer bob carol 1",
        "withdraw dave 5",
        "deposit carol abc",
        "audit alice 1",
        "transfer bob carol 7550",
    ]
    ledger = Ledger()
    moved = 0
    for line in lines:
        try:
            n = ledger.apply(line)
        except LedgerError as err:
            print(f"rejected: {line} -> {err}")
            continue
        moved += n
    for who in sorted(ledger.balances):
        print(f"{who}: {money(ledger.balances[who])}")
    print(f"applied {ledger.applied} of {len(lines)}, moved {money(moved)}")


main()
