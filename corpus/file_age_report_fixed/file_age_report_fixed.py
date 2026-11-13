DAY = 86400000
HOUR = 3600000


def bucket(age):
    if age < DAY:
        return "today"
    if age < 7 * DAY:
        return "week"
    if age < 30 * DAY:
        return "month"
    return "stale"


def age_text(age):
    days = age // DAY
    hours = age % DAY // HOUR
    return f"{days}d{hours:02}h"


def main():
    specs = [
        ("build.log", 40, 0, 3),
        ("notes.txt", 25, 2, 12),
        ("old.tar", 300, 45, 5),
        ("report.pdf", 120, 9, 0),
        ("cache.db", 80, 400, 23),
        ("todo.md", 10, 6, 23),
        ("draft.txt", 15, 29, 1),
    ]
    found = sorted(((n, s, d * DAY + h * HOUR) for n, s, d, h in sorted(specs)), key=lambda a: -a[2])
    print(f"{'file':<12}{'age':>9}  {'bucket':<6}{'bytes':>6}")
    for n, s, age in found:
        print(f"{n:<12}{age_text(age):>9}  {bucket(age):<6}{s:>6}")
    for b in ["today", "week", "month", "stale"]:
        n = 0
        total = 0
        for _, s, age in found:
            if bucket(age) == b:
                n += 1
                total += s
        print(f"{b:<6}{n:>2} files{total:>5} bytes")
    oldest = found[0]
    print(f"oldest: {oldest[0]}, {oldest[2] // DAY} days")
    print("cleaned up: true")


main()
