from collections import Counter
from pathlib import Path

LEVELS = ["INFO", "WARN", "ERROR"]


def parse_line(line):
    words = line.split(" ")
    if len(words) < 4:
        return None
    return words[1].split(":")[0], words[2], " ".join(words[3:])


def top_errors(counts, n):
    return sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))[:n]


def main():
    text = (Path(__file__).resolve().parents[5] / "examples/data/logscan.log").read_text()
    per_hour = Counter()
    errors = Counter()
    skipped = 0
    for line in text.splitlines():
        entry = parse_line(line)
        if entry is None:
            skipped += 1
            continue
        hour, level, message = entry
        per_hour[hour, level] += 1
        if level == "ERROR":
            errors[message] += 1
    print(f"hour {LEVELS[0]:>6} {LEVELS[1]:>6} {LEVELS[2]:>6}")
    for hour in sorted({h for h, _ in per_hour}):
        cells = " ".join(f"{per_hour[hour, level]:>6}" for level in LEVELS)
        print(f"{hour}:00 {cells}")
    print("top errors:")
    for message, count in top_errors(errors, 3):
        print(f"{count:>3}  {message}")
    print(f"skipped {skipped} malformed lines")


def test_parses_a_line():
    assert parse_line("2026-10-12 09:15:00 WARN slow query") == ("09", "WARN", "slow query")
    assert parse_line("garbled") is None


def test_ranks_by_count_then_message():
    assert [m for m, _ in top_errors({"b": 2, "a": 2, "c": 5}, 2)] == ["c", "a"]


if __name__ == "__main__":
    main()
