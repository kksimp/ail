from collections import Counter
from datetime import datetime, timezone
from pathlib import Path


def main():
    lines = (Path(__file__).resolve().parents[5] / "examples/data/events.csv").read_text().splitlines()
    per_day = Counter()
    kinds = Counter()
    for line in lines[1:]:
        ts, kind = line.split(",")
        day = datetime.fromtimestamp(int(ts), tz=timezone.utc).strftime("%Y-%m-%d")
        per_day[day] += 1
        kinds[kind] += 1
    for day in sorted(per_day):
        print(f"{day}  {per_day[day]} events")
    print("kinds: " + ", ".join(sorted(kinds)))


if __name__ == "__main__":
    main()
