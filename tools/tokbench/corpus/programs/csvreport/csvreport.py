import csv
from collections import Counter
from pathlib import Path

DATA = Path(__file__).resolve().parents[5] / "examples/data"


def parse_row(row):
    if len(row) != 4:
        return None
    region, product, units, price = row
    try:
        return region, product, int(units), float(price)
    except ValueError:
        return None


def main():
    with open(DATA / "sales.csv", newline="") as f:
        rows = list(csv.reader(f))[1:]
    revenue, units, by_product = Counter(), Counter(), Counter()
    bad = 0
    for row in rows:
        sale = parse_row(row)
        if sale is None:
            bad += 1
            continue
        region, product, n, price = sale
        revenue[region] += n * price
        units[region] += n
        by_product[product] += n * price
    print(f"{'region':<8}{'units':>6}{'revenue':>10}")
    for region in sorted(revenue):
        print(f"{region:<8}{units[region]:>6}{revenue[region]:>10.2f}")
    best = max(by_product, key=by_product.get)
    print(f"total {sum(revenue.values()):.2f}, best {best}, {bad} bad rows")


def test_parses_rows():
    assert parse_row(["north", "widget", "2", "1.5"]) == ("north", "widget", 2, 1.5)
    assert parse_row(["north", "widget", "two", "1.5"]) is None
    assert parse_row(["north"]) is None


if __name__ == "__main__":
    main()
