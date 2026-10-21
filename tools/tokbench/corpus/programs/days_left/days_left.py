import datetime
import sys
from pathlib import Path

DATA = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "agent-data"


def main():
    today = datetime.date(2026, 10, 21)
    text = (DATA / "dates.txt").read_text()
    for line in text.splitlines():
        left = (datetime.date.fromisoformat(line) - today).days
        if left < 0:
            print(f"{line}: {-left} days late")
        else:
            print(f"{line}: {left} days left")


if __name__ == "__main__":
    main()
