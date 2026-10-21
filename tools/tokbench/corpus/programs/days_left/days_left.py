import datetime
from pathlib import Path


def main():
    today = datetime.date(2026, 10, 21)
    text = (Path(__file__).resolve().parents[5] / "examples/agent-data/dates.txt").read_text()
    for line in text.splitlines():
        left = (datetime.date.fromisoformat(line) - today).days
        if left < 0:
            print(f"{line}: {-left} days late")
        else:
            print(f"{line}: {left} days left")


if __name__ == "__main__":
    main()
