class CouponError(Exception):
    pass


def money(cents):
    return f"{cents // 100}.{cents % 100:0>2}"


def amount(kind, total):
    if kind[0] == "pct":
        return total * kind[1] // 100
    if kind[0] == "cap":
        return min(total * kind[1] // 100, kind[2])
    return min(kind[1], total)


class Engine:
    def __init__(self):
        self.coupons = {}
        self.used = {}

    def redeem(self, code, total, day):
        if code not in self.coupons:
            raise CouponError(f"unknown coupon {code}")
        kind, min_total, expires, max_uses = self.coupons[code]
        if day > expires:
            raise CouponError(f"{code} expired on day {expires}")
        if total < min_total:
            raise CouponError(f"{code} needs a total of at least {min_total}")
        n = self.used.get(code, 0)
        if n >= max_uses:
            raise CouponError(f"{code} has no uses left")
        self.used[code] = n + 1
        return amount(kind, total)

    def price(self, oid, total, day, codes):
        running = total
        applied = 0
        for code in codes.split(","):
            if applied >= 2:
                print(f"  {code}: skipped, two coupons per order")
                continue
            try:
                d = self.redeem(code, running, day)
            except CouponError as err:
                print(f"  {code}: {err}")
                continue
            running -= d
            applied += 1
            print(f"  {code}: -{money(d)}")
        print(f"order {oid}: {money(total)} -> {money(running)}")


def main():
    engine = Engine()
    engine.coupons["TENOFF"] = (("pct", 10), 0, 30, 3)
    engine.coupons["BIG20"] = (("cap", 20, 2500), 10000, 30, 2)
    engine.coupons["FIVER"] = (("fix", 500), 2000, 10, 1)
    engine.price(1, 4000, 5, "TENOFF,FIVER")
    engine.price(2, 20000, 6, "BIG20,TENOFF,FIVER")
    engine.price(3, 3000, 7, "FIVER,NOPE,BIG20")
    engine.price(4, 15000, 12, "FIVER,BIG20")
    engine.price(5, 800, 12, "TENOFF")
    engine.price(6, 9000, 13, "TENOFF")
    for code in ["BIG20", "FIVER", "TENOFF"]:
        print(f"{code} used {engine.used.get(code, 0)}")


main()
