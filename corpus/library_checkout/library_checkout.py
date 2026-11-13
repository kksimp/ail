class LibraryError(Exception):
    pass


LOAN_DAYS = 14
MAX_LOANS = 2
FEE_PER_DAY = 25


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


class Library:
    def __init__(self, stock):
        self.stock = stock
        self.loans = {}
        self.fees = 0

    def held_by(self, member):
        prefix = f"{member}/"
        n = 0
        for key in self.loans:
            if key.startswith(prefix):
                n += 1
        return n

    def apply(self, line):
        parts = line.split(" ")
        if len(parts) != 4:
            raise LibraryError(f"bad command '{line}'")
        member = parts[1]
        title = parts[2]
        try:
            day = int(parts[3])
        except ValueError:
            raise LibraryError(f"bad command '{line}'")
        key = f"{member}/{title}"
        if parts[0] == "out":
            if title not in self.stock:
                raise LibraryError(f"no such book {title}")
            copies = self.stock[title]
            if copies == 0:
                raise LibraryError(f"no copies of {title} left")
            if self.held_by(member) >= MAX_LOANS:
                raise LibraryError(f"{member} already holds {MAX_LOANS} books")
            self.stock[title] = copies - 1
            self.loans[key] = day + LOAN_DAYS
            return f"{member} borrowed {title}, due day {day + LOAN_DAYS}"
        if parts[0] != "back":
            raise LibraryError(f"bad command '{line}'")
        if key not in self.loans:
            raise LibraryError(f"{member} does not hold {title}")
        due = self.loans.pop(key)
        self.stock[title] += 1
        late = day - due if day > due else 0
        self.fees += late * FEE_PER_DAY
        return f"{member} returned {title}, {late} days late, fee {money(late * FEE_PER_DAY)}"


def main():
    lib = Library({"dune": 1, "emma": 2, "ulysses": 1})
    script = [
        "out ann dune 1",
        "out bob dune 2",
        "out ann emma 3",
        "out ann ulysses 4",
        "out bob hamlet 5",
        "back bob emma 10",
        "back ann dune 20",
        "out bob dune 21",
        "back ann emma 17",
        "borrow cy emma 1",
        "out cy emma x",
    ]
    for line in script:
        try:
            msg = lib.apply(line)
        except LibraryError as err:
            print(f"{line}: ERROR {err}")
            continue
        print(f"{line}: {msg}")
    for t in sorted(lib.stock):
        print(f"stock {t}: {lib.stock[t]}")
    open_loans = sorted(lib.loans)
    print("open loans: " + ", ".join(open_loans))
    print(f"fees collected: {money(lib.fees)}")


main()
