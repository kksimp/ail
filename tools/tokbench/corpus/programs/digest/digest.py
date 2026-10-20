import sys
import urllib.error
import urllib.request
from collections import Counter
from pathlib import Path


def load(url):
    try:
        with urllib.request.urlopen(url) as r:
            return r.read().decode()
    except urllib.error.URLError as err:
        kind = "refused" if isinstance(err.reason, ConnectionRefusedError) else "other"
        print(f"offline ({kind}), using the local copy")
        return (Path(__file__).resolve().parents[5] / "examples/data" / url.rsplit("/", 1)[-1]).read_text()


def totals(csv):
    units = Counter()
    for line in csv.splitlines()[1:]:
        cells = line.split(",")
        try:
            units[cells[1]] += int(cells[2])
        except ValueError:
            units[cells[1]] += 0
    return units


def main():
    url = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:1/sales.csv"
    units = totals(load(url))
    for p in sorted(units, key=lambda p: -units[p]):
        print(f"{p:<8}{units[p]:>4}")
    print(f"{len(units)} products, {sum(units.values())} units")


def test_sums_units():
    assert totals("p,q,u\na,x,2\nb,x,3\nc,y,4") == {"x": 5, "y": 4}


if __name__ == "__main__":
    main()
