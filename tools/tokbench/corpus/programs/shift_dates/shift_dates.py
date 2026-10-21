import datetime
from pathlib import Path


def main():
    text = (Path(__file__).resolve().parents[5] / "examples/agent-data/dates.txt").read_text()
    for line in text.splitlines():
        d = datetime.date.fromisoformat(line)
        later = d + datetime.timedelta(days=7)
        print(f"{d.isoformat()} is a {d.isoweekday()}, a week later {later.isoformat()}")


if __name__ == "__main__":
    main()
