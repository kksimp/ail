GOAL = "123456780"


def slide(state, d):
    deltas = [-3, 3, -1, 1]
    cs = list(state)
    b = cs.index('0')
    row, col = b // 3, b % 3
    if d == 0 and row == 0:
        return None
    if d == 1 and row == 2:
        return None
    if d == 2 and col == 0:
        return None
    if d == 3 and col == 2:
        return None
    cs[b], cs[b + deltas[d]] = cs[b + deltas[d]], cs[b]
    return "".join(cs)


def solvable(state):
    ds = [int(c) for c in state if c != '0']
    inv = 0
    for i in range(len(ds)):
        for j in range(i + 1, len(ds)):
            if ds[i] > ds[j]:
                inv += 1
    return inv % 2 == 0


def solve(start):
    if not solvable(start):
        return None
    letters = ["U", "D", "L", "R"]
    seen = {start}
    states = [start]
    parent = [-1]
    how = [""]
    head = 0
    while head < len(states):
        cur = states[head]
        if cur == GOAL:
            break
        for d in range(4):
            nxt = slide(cur, d)
            if nxt is None:
                continue
            if nxt not in seen:
                seen.add(nxt)
                states.append(nxt)
                parent.append(head)
                how.append(letters[d])
        head += 1
    if head >= len(states):
        return None
    moves = []
    at = head
    while parent[at] != -1:
        moves.append(how[at])
        at = parent[at]
    moves.reverse()
    return "".join(moves)


def replay(start, moves):
    letters = "UDLR"
    cur = start
    for ch in moves:
        d = letters.find(ch)
        if d < 0:
            d = 0
        nx = slide(cur, d)
        cur = nx if nx is not None else cur
    return cur


def board(state):
    cs = list(state)
    return [" ".join(cs[0:3]), " ".join(cs[3:6]), " ".join(cs[6:9])]


def report(start):
    print(f"start {start}")
    for line in board(start):
        print(f"  {line}")
    if not solvable(start):
        print("  unsolvable (odd inversion count)")
        return
    answer = solve(start)
    if answer is None:
        answer = ""
    print(f"  {len(answer)} moves: {answer}")
    print(f"  replay reaches goal: {'true' if replay(start, answer) == GOAL else 'false'}")


def main():
    report("724506831")
    report("123405786")
    report("102453786")
    report("213456780")
    report("120453786")


def tests():
    assert slide("123456780", 0) == "123450786"
    assert slide("123456780", 2) == "123456708"
    assert slide("123456780", 1) is None
    assert solvable(GOAL) and not solvable("213456780")
    assert solve("123456708") == "R"
    assert len(solve("123450786")) == 1


tests()
main()
