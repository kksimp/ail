class ClickLogger:
    def __init__(self, margin):
        self.margin = margin

    def label(self):
        return "clicks"

    def wants(self, ev):
        return ev[0] == "click" and ev[1] >= self.margin and ev[2] >= self.margin

    def handle(self, ev):
        return f"click at {ev[1]},{ev[2]}"


class KeyFilter:
    def __init__(self, min_code):
        self.min_code = min_code

    def label(self):
        return "keys"

    def wants(self, ev):
        return ev[0] == "key" and ev[1] >= self.min_code

    def handle(self, ev):
        return f"key {ev[1]}"


class Heartbeat:
    def __init__(self, every):
        self.every = every

    def label(self):
        return "beat"

    def wants(self, ev):
        return ev[0] == "tick"

    def handle(self, ev):
        return f"tick every {self.every}"


handlers = []
delivered = 0
dropped = 0


def publish(ev):
    global delivered, dropped
    lines = []
    hits = 0
    for h in handlers:
        if h.wants(ev):
            lines.append(f"[{h.label()}] {h.handle(ev)}")
            hits += 1
    if hits == 0:
        dropped += 1
    delivered += hits
    return lines


def show(ev):
    if ev[0] == "click":
        return f"Click(x: {ev[1]}, y: {ev[2]})"
    if ev[0] == "key":
        return f"Key(code: {ev[1]})"
    return "Tick"


handlers.append(ClickLogger(10))
handlers.append(KeyFilter(65))
handlers.append(Heartbeat(5))
handlers.append(KeyFilter(90))
events = [("click", 50, 20), ("click", 3, 4), ("key", 66), ("tick",), ("key", 95), ("key", 10)]
for ev in events:
    lines = publish(ev)
    if not lines:
        print(f"{show(ev)} -> nobody listened")
    for line in lines:
        print(line)
print(f"delivered {delivered}, dropped {dropped}")
