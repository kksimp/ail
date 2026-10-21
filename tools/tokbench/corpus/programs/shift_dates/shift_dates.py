import datetime
import sys
from pathlib import Path

DATA = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent / "agent-data"


def main():
    text = (DATA / "dates.txt").read_text()
    for line in text.splitlines():
        d = datetime.date.fromisoformat(line)
        later = d + datetime.timedelta(days=7)
        print(f"{d.isoformat()} is a {d.isoweekday()}, a week later {later.isoformat()}")


if __name__ == "__main__":
    main()
