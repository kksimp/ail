def day_names():
    return ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]


def month_names():
    return ["January", "February", "March", "April", "May", "June", "July",
            "August", "September", "October", "November", "December"]


def offsets():
    return [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]


def is_leap(year):
    return (year % 4 == 0 and year % 100 != 0) or year % 400 == 0


def days_in_month(year, month):
    if month == 2:
        return 29 if is_leap(year) else 28
    if month == 4 or month == 6 or month == 9 or month == 11:
        return 30
    return 31


def weekday(year, month, day):
    y = year - 1 if month < 3 else year
    return (y + y // 4 - y // 100 + y // 400 + offsets()[month - 1] + day) % 7


def render(year, month):
    title = f"{month_names()[month - 1]} {year}"
    pad = (20 - len(title)) // 2
    lines = [title.rjust(pad + len(title)), "Su Mo Tu We Th Fr Sa"]
    cells = []
    for _ in range(weekday(year, month, 1)):
        cells.append("  ")
    for day in range(1, days_in_month(year, month) + 1):
        cells.append(f"{day:>2}")
    start = 0
    while start < len(cells):
        stop = min(start + 7, len(cells))
        lines.append(" ".join(cells[start:stop]))
        start += 7
    return lines


def friday_13ths(year):
    out = []
    for month in range(1, 13):
        if weekday(year, month, 13) == 5:
            out.append(month_names()[month - 1][0:3])
    return out


def main():
    for line in render(2024, 2):
        print(line)
    print("")
    for line2 in render(2026, 11):
        print(line2)
    print("")
    for line3 in render(2000, 2):
        print(line3)
    print("")
    print(f"2024-02-29 fell on a {day_names()[weekday(2024, 2, 29)]}")
    print(f"2026-11-13 is a {day_names()[weekday(2026, 11, 13)]}")
    print(f"2000-01-01 was a {day_names()[weekday(2000, 1, 1)]}")
    for year in [2023, 2024, 2026]:
        print(f"Friday the 13th in {year}: {' '.join(friday_13ths(year))}")
    print(f"leap years 1896..1912: {' '.join(str(y) for y in range(1896, 1913) if is_leap(y))}")


main()
