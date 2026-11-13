DAYS = 2
PERIODS = 3
N = DAYS * PERIODS


class TimetableError(Exception):
    pass


def E(cls, subject, teacher, room):
    return {"class": cls, "subject": subject, "teacher": teacher, "room": room}


RULES = [("teacher", "teacher"), ("room", "room"), ("class", "class")]


class Timetable:
    def __init__(self):
        self.slots = [[] for _ in range(N)]

    def conflict(self, e, slot):
        for other in self.slots[slot]:
            for name, field in RULES:
                if other[field] == e[field]:
                    return name
        return ""

    def place(self, e, slot):
        if slot < 0 or slot >= N:
            raise TimetableError(f"slot {slot} does not exist")
        why = self.conflict(e, slot)
        if why != "":
            raise TimetableError(f"slot {slot} clashes on {why}")
        self.slots[slot].append(e)

    def auto(self, e):
        for s in range(N):
            if self.conflict(e, s) == "":
                self.slots[s].append(e)
                return s
        raise TimetableError(f"no free slot for {e['class']} {e['subject']}")

    def week(self, cls):
        cells = []
        for s in range(N):
            cell = "---"
            for e in self.slots[s]:
                if e["class"] == cls:
                    cell = e["subject"][:3]
            cells.append(cell)
        days = [" ".join(cells[d * PERIODS:(d + 1) * PERIODS]) for d in range(DAYS)]
        return f"{cls}: " + " | ".join(days)


t = Timetable()
plan = [
    [E("9A", "math", "Kim", "R1"), E("9B", "math", "Kim", "R1")],
    [E("9A", "art", "Lee", "R2"), E("9B", "art", "Lee", "R2")],
    [E("9A", "pe", "Ray", "gym"), E("9B", "pe", "Ray", "gym")],
]
hours = [3, 2, 2]
unplaced = 0
placed = []
for i in range(len(plan)):
    for e in plan[i]:
        for h in range(hours[i]):
            try:
                slot = t.auto(e)
            except TimetableError as err:
                print(f"{err}")
                unplaced += 1
                continue
            placed.append(slot)
print(t.week("9A"))
print(t.week("9B"))
print(f"placed {len(placed)}, unplaced {unplaced}")
pin = E("9C", "music", "Kim", "R3")
for sl in (0, 9):
    try:
        t.place(pin, sl)
    except TimetableError as err:
        print(f"pin failed: {err}")
