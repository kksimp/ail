class PhoneError(Exception):
    pass


REGIONS = [("1", 10, 10), ("44", 10, 10), ("49", 10, 11), ("33", 9, 9)]


def digits_of(s):
    return "".join(c for c in s if "0" <= c <= "9")


def split_ext(raw):
    low = raw.lower()
    e = low.find("ext")
    x = low.find("x")
    at = e if e >= 0 else x
    if at < 0:
        return (low, "")
    return (low[:at], digits_of(low[at:]))


def us_phone(national, ext):
    if national.startswith("0") or national.startswith("1"):
        raise PhoneError(f"area code {national[:3]} cannot start with 0 or 1")
    return ("1", national, ext)


def normalize(raw):
    main_part, ext = split_ext(raw)
    digits = digits_of(main_part)
    if main_part.strip().startswith("+"):
        for code, lo, hi in REGIONS:
            if digits.startswith(code):
                national = digits[len(code):]
                if len(national) < lo or len(national) > hi:
                    raise PhoneError(f"{len(national)} digits do not fit the country")
                return (code, national, ext)
        raise PhoneError(f"unknown country code in +{digits[:2]}")
    if len(digits) == 11 and digits.startswith("1"):
        return us_phone(digits[1:11], ext)
    if len(digits) == 10:
        return us_phone(digits, ext)
    if len(digits) < 10:
        raise PhoneError(f"only {len(digits)} digits")
    raise PhoneError(f"{len(digits)} digits do not fit the country")


def e164(p):
    return f"+{p[0]}{p[1]}"


def pretty(p):
    country, national, ext = p
    base = f"+{country} {national}"
    if country == "1":
        base = f"+1 ({national[0:3]}) {national[3:6]}-{national[6:10]}"
    if ext == "":
        return base
    return f"{base} x{ext}"


def main():
    contacts = [
        ("Ann", "(555) 123-4567"),
        ("Bob", "555.123.4567 ext. 22"),
        ("Cy", "1-555-123-4567"),
        ("Dee", "+44 20 7946 0958"),
        ("Eli", "020 7946 0958"),
        ("Fay", "+49 30 1234 5678 x5"),
        ("Gus", "123-4567"),
        ("Hal", "(055) 123-4567"),
        ("Ivy", "+81 3 1234 5678"),
        ("Jo", "+33 1 23 45 67 89"),
        ("Kit", "+1 555 123 456"),
    ]
    groups = {}
    for who, num in contacts:
        try:
            ph = normalize(num)
        except PhoneError as err:
            print(f"{who:<4} {num:<22} error: {err}")
            continue
        print(f"{who:<4} {num:<22} {e164(ph)} | {pretty(ph)}")
        groups.setdefault(e164(ph), []).append(who)
    print("duplicates:")
    for key in sorted(groups):
        names = groups[key]
        if len(names) > 1:
            print(f"  {key}: {', '.join(names)}")


main()
