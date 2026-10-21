import json
import re


def main():
    digits = re.compile(r"\d+")
    log = "order 66 shipped to 10.2 and 7 more"
    print(f"has digits: {digits.search(log) is not None}".lower())
    for m in digits.finditer(log):
        print(f"number {m.group(0)} at character {m.start()}")
    date = re.compile(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})")
    print(date.sub(lambda m: f"{m.group('d')}/{m.group('m')}/{m.group('y')}", "due 2026-10-20"))
    comma = re.compile(r"\s*,\s*")
    print(json.dumps(comma.split("a , b,c")))


if __name__ == "__main__":
    main()
