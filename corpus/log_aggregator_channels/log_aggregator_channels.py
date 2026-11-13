def main():
    sources = [
        [
            "10:00:01 INFO web GET /index 200",
            "10:00:02 INFO web GET /about 200",
            "10:00:03 ERROR web upstream timeout",
            "10:00:04 WARN web slow response 1200ms",
        ],
        [
            "10:00:01 INFO db checkpoint done",
            "10:00:02 ERROR db deadlock detected",
            "10:00:05 ERROR db disk almost full",
            "garbage",
        ],
        [
            "10:00:01 DEBUG auth token refreshed",
            "10:00:03 WARN auth 3 failed logins",
            "10:00:06 ERROR auth locked account",
            "10:00:07 INFO auth login ok",
            "10:00:08 INFO",
        ],
    ]
    levels, services, errors, bad = {}, {}, [], 0
    for lines in sources:
        for line in lines:
            parts = line.split(" ", 3)
            if len(parts) < 4:
                bad += 1
                continue
            levels[parts[1]] = levels.get(parts[1], 0) + 1
            services[parts[2]] = services.get(parts[2], 0) + 1
            if parts[1] == "ERROR":
                errors.append(f"{parts[2]}: {parts[3]}")
    for level in ["ERROR", "WARN", "INFO", "DEBUG"]:
        print(f"{level:<6}{levels.get(level, 0):>3}")
    for name in sorted(services):
        print(f"{name:<6}{services[name]:>3} lines")
    print("errors:")
    for e in sorted(errors):
        print(f"  {e}")
    print(f"malformed: {bad}")
    print(f"total lines: {sum(levels.values()) + bad}")


main()
