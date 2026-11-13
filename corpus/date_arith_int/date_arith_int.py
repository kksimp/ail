import re


def to_int(s):
    return int(s) if re.fullmatch(r"-?[0-9]+", s) else None


def is_leap(y):
    return (y % 4 == 0 and y % 100 != 0) or y % 400 == 0


def days_in_month(y, m):
    if m == 2:
        return 29 if is_leap(y) else 28
    if m in (4, 6, 9, 11):
        return 30
    return 31


def valid(dt):
    y, m, d = dt
    return y >= 1970 and 1 <= m <= 12 and 1 <= d <= days_in_month(y, m)


def to_days(dt):
    y, m, d = dt
    yy = y - 1 if m <= 2 else y
    era = yy // 400
    yoe = yy - era * 400
    mp = (m + 9) % 12
    doy = (153 * mp + 2) // 5 + d - 1
    doe = yoe * 365 + yoe // 4 - yoe // 100 + doy
    return era * 146097 + doe - 719468


def from_days(days):
    z = days + 719468
    era = z // 146097
    doe = z - era * 146097
    yoe = (doe - doe // 1460 + doe // 36524 - doe // 146096) // 365
    doy = doe - (365 * yoe + yoe // 4 - yoe // 100)
    mp = (5 * doy + 2) // 153
    d = doy - (153 * mp + 2) // 5 + 1
    m = mp + 3 if mp < 10 else mp - 9
    y = yoe + era * 400
    return (y + 1 if m <= 2 else y, m, d)


def show(dt):
    return f"{dt[0]}-{dt[1]:02}-{dt[2]:02}"


def parse(text):
    parts = text.split("-")
    if len(parts) != 3:
        return None
    nums = [to_int(p) for p in parts]
    if None in nums:
        return None
    dt = tuple(nums)
    if valid(dt):
        return dt
    return None


def weekday(dt):
    return ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"][to_days(dt) % 7]


def day_of_year(dt):
    return to_days(dt) - to_days((dt[0], 1, 1)) + 1


def add_days(dt, n):
    return from_days(to_days(dt) + n)


inputs = ["1970-01-01", "2000-02-29", "2024-03-01", "2023-02-29", "2024-12-31", "1969-12-31", "2100-02-29", "bad"]
for t in inputs:
    dt = parse(t)
    if dt is None:
        print(f"{t}: invalid")
        continue
    print(f"{show(dt)} {weekday(dt)} day {day_of_year(dt)} of {366 if is_leap(dt[0]) else 365} days={to_days(dt)}")
start = (2024, 1, 31)
for n in [1, 29, 30, 366, 10000]:
    print(f"{show(start)} + {n} = {show(add_days(start, n))}")
a = (2000, 1, 1)
b = (2024, 11, 13)
print(f"days from {show(a)} to {show(b)}: {to_days(b) - to_days(a)}")
