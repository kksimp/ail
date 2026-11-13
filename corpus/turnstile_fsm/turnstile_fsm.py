LABEL = {"L": "locked", "U": "unlocked", "W": "warned", "A": "alarm"}
TABLE = {
    "L": {"C": "U", "P": "W", "R": "L"},
    "U": {"C": "U", "P": "L", "R": "L"},
    "W": {"C": "U", "P": "A", "R": "L"},
    "A": {"C": "A", "P": "A", "R": "L"},
}


def step(s, i):
    return TABLE[s][i]


def table_row(s):
    a = LABEL[step(s, "C")]
    b = LABEL[step(s, "P")]
    c = LABEL[step(s, "R")]
    return f"{LABEL[s]:<9} {a:<9} {b:<9} {c}"


def run(script):
    cur = "L"
    trace = "L"
    alarms = 0
    passes = 0
    skipped = 0
    for ch in script:
        if ch not in "CPR":
            skipped += 1
            continue
        nxt = step(cur, ch)
        if nxt == "A" and cur != "A":
            alarms += 1
        if cur == "U" and ch == "P":
            passes += 1
        trace += nxt
        cur = nxt
    return f"{trace} final={LABEL[cur]} passes={passes} alarms={alarms} skipped={skipped}"


print("state     coin      push      reset")
for s in "LUWA":
    print(table_row(s))
for script in ["CP", "PPCP", "PPPCR", "PPPCCRCP", "CPCPCP", "XCPX", ""]:
    print(f"{script:<9} {run(script)}")
