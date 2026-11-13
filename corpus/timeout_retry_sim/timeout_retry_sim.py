def latency_for(lats, attempt):
    return lats[min(attempt - 1, len(lats) - 1)]


def simulate(lats, timeout, retries, deadline, pause):
    clock = 0
    log = []
    for attempt in range(1, retries + 2):
        room = deadline - clock
        if room <= 0:
            log.append(f"t={clock:>3} deadline hit before attempt {attempt}")
            return ("deadline", clock, attempt - 1, log)
        lat = latency_for(lats, attempt)
        limit = min(timeout, room)
        if lat <= limit:
            clock += lat
            log.append(f"t={clock:>3} attempt {attempt} ok after {lat} ms")
            return ("ok", clock, attempt, log)
        clock += limit
        log.append(f"t={clock:>3} attempt {attempt} timed out ({limit} ms)")
        if attempt <= retries and clock < deadline:
            clock += pause * attempt
            log.append(f"t={clock:>3} backoff {pause * attempt} ms")
    return ("failed", clock, retries + 1, log)


def show(name, r):
    print(f"{name}: {r[0]} at t={r[1]} after {r[2]} attempts")
    for line in r[3]:
        print(f"  {line}")


def main():
    show("fast", simulate([30], 100, 3, 400, 20))
    show("slow start", simulate([250, 180, 60], 100, 3, 400, 20))
    show("always slow", simulate([500], 100, 3, 400, 20))
    show("deadline", simulate([200], 150, 5, 300, 30))


main()
