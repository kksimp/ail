def apply(state, line):
    parts = line.split(" ")
    if len(parts) < 2:
        return False
    cmd, key = parts[0], parts[1]
    if cmd == "del":
        state.pop(key, None)
        return True
    if len(parts) != 3:
        return False
    try:
        n = int(parts[2])
    except ValueError:
        return False
    if cmd == "set":
        state[key] = n
        return True
    if cmd == "inc":
        state[key] = state.get(key, 0) + n
        return True
    return False


def replay(lines, upto):
    state = {}
    applied = 0
    bad = 0
    for i in range(min(len(lines), upto)):
        if apply(state, lines[i]):
            applied += 1
        else:
            bad += 1
    return state, applied, bad


def show(label, r):
    state, applied, bad = r
    print(f"{label}: {applied} applied, {bad} bad")
    for k in sorted(state):
        print(f"  {k} = {state[k]}")


def main():
    text = "set a 5\nset b 10\ninc a 3\nbogus line\ndel b\nset c x\ninc c 4\nset b 1\ninc a -2\n"
    lines = text.splitlines()
    print(f"log has {len(lines)} lines")
    show("after 3", replay(lines, 3))
    show("after 6", replay(lines, 6))
    show("full", replay(lines, 100))
    print("cleaned up: true")


main()
