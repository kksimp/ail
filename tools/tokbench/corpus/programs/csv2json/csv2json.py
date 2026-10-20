import json
from pathlib import Path


def main():
    lines = (Path(__file__).resolve().parents[5] / "examples/data/sales.csv").read_text().splitlines()
    header = lines[0].split(",")
    rows = []
    for line in lines[1:]:
        row = {}
        for key, cell in zip(header, line.split(",")):
            if key in ("units", "price"):
                try:
                    row[key] = float(cell)
                except ValueError:
                    row[key] = 0.0
            else:
                row[key] = cell
        rows.append(row)
    print(json.dumps(rows, separators=(",", ":")))


if __name__ == "__main__":
    main()
