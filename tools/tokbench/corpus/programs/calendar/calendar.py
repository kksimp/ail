from datetime import datetime, timedelta, timezone
from zoneinfo import ZoneInfo


def iso_utc(dt):
    return dt.strftime("%Y-%m-%dT%H:%M:%S") + "Z"


def main():
    print(f"2024 leap: {is_leap(2024)}, 2100 leap: {is_leap(2100)}".lower())
    print(f"February 2024 has {days_in_month(2024, 2)} days")

    d = datetime(2024, 2, 28)
    print(f"{d.date().isoformat()} is weekday {d.isoweekday()} (1 = Monday), day {d.timetuple().tm_yday} of the year")
    print(f"two days later: {(d + timedelta(days=2)).date().isoformat()}")
    print(f"31 January plus one month: {add_months(datetime(2024, 1, 31), 1).date().isoformat()}")

    ny = ZoneInfo("America/New_York")
    t = datetime(2024, 3, 10, 1, 30, 0, tzinfo=ny)
    print(f"start: {t.strftime('%Y-%m-%d %H:%M:%S %z')}")
    elapsed = t.astimezone(timezone.utc) + timedelta(hours=1)
    print(f"one elapsed hour later: {elapsed.astimezone(ny).strftime('%H:%M %z')}")
    print(f"one wall-clock day later: {(t + timedelta(days=1)).strftime('%Y-%m-%d %H:%M %z')}")

    noon = datetime.fromtimestamp(1719835200, tz=timezone.utc)
    print(f"noon UTC is {iso_utc(noon)}")
    paris = noon.astimezone(ZoneInfo("Europe/Paris"))
    print(f"in Paris: {paris.strftime('%H:%M %z')}")

    parsed = datetime.fromisoformat("2024-07-01T14:00:00+02:00")
    print(f"ISO round trip: {parsed.isoformat()}")
    print(f"same instant as noon UTC: {str(parsed == noon).lower()}")


def is_leap(y):
    return y % 4 == 0 and (y % 100 != 0 or y % 400 == 0)


def days_in_month(y, m):
    if m == 2:
        return 29 if is_leap(y) else 28
    return 30 if m in (4, 6, 9, 11) else 31


def add_months(d, n):
    month = d.month - 1 + n
    year = d.year + month // 12
    month = month % 12 + 1
    day = min(d.day, days_in_month(year, month))
    return d.replace(year=year, month=month, day=day)


if __name__ == "__main__":
    main()
