def sample():
    return [
        "2026-01-05 09:58:01 INFO web: started",
        "2026-01-05 10:00:07 ERROR db: timeout after 30s",
        "2026-01-05 10:00:09 WARN web: slow request",
        "garbled line",
        "2026-01-05 10:15:44 ERROR db: connection reset",
        "2026-01-05 10:16:00 INFO cache: warmed 120 keys",
        "2026-01-05 11:02:10 ERROR web: 500 on /checkout",
        "2026-01-05 11:02:11 DEBUG web: ignored level",
        "2026-01-05 11:30:00 ERROR db: deadlock detected",
    ]


def parse_line(text):
    parts = text.split(" ")
    if len(parts) < 4:
        return None
    level = parts[2]
    if level not in ["INFO", "WARN", "ERROR"]:
        return None
    return (parts[1][0:2], level, parts[3].replace(":", ""), " ".join(parts[4:]))


def main():
    levels = {}
    hours = {}
    errors = {}
    bad = 0
    last_error = ""
    for text in sample():
        line = parse_line(text)
        if line is None:
            bad += 1
            continue
        hour, level, comp, msg = line
        levels[level] = levels.get(level, 0) + 1
        hours[hour] = hours.get(hour, 0) + 1
        if level == "ERROR":
            errors[comp] = errors.get(comp, 0) + 1
            last_error = msg
    print(f"unparsed: {bad}")
    for l in ["ERROR", "WARN", "INFO"]:
        print(f"{l:<6}{levels.get(l, 0):>3}")
    busiest = sorted(sorted(hours), key=lambda h: -hours[h])[0]
    print(f"busiest hour: {busiest}:00 ({hours[busiest]} lines)")
    for c in sorted(sorted(errors), key=lambda k: -errors[k]):
        print(f"errors in {c}: {errors[c]}")
    print(f"last error: {last_error}")


main()
