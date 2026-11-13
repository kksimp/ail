def hanoi(n, src, dst, via, moves):
    if n == 0:
        return
    hanoi(n - 1, src, via, dst, moves)
    moves.append((n - 1, src, dst))
    hanoi(n - 1, via, dst, src, moves)


def legal(pos, m):
    disk, src, dst = m
    if pos[disk] != src:
        return False
    for j in range(disk):
        if pos[j] == src or pos[j] == dst:
            return False
    return True


def replay(n, moves):
    pos = [0] * n
    for m in moves:
        if not legal(pos, m):
            return False
        pos[m[0]] = m[2]
    for p in pos:
        if p != 2:
            return False
    return True


def ruler(k):
    v, z = k, 0
    while v % 2 == 0:
        v //= 2
        z += 1
    return z


def peg_name(p):
    return ["A", "B", "C"][p]


def main():
    moves = []
    hanoi(3, 0, 2, 1, moves)
    for i, m in enumerate(moves):
        print(f"{i + 1:>2}: disk {m[0] + 1} {peg_name(m[1])} -> {peg_name(m[2])}")
    print("disks  moves  valid")
    for n in range(1, 11):
        sol = []
        hanoi(n, 0, 2, 1, sol)
        print(f"{n:>5}  {len(sol):>5}  {'true' if replay(n, sol) else 'false'}")


def tests():
    for n in range(9):
        a = []
        hanoi(n, 0, 2, 1, a)
        assert len(a) == (1 << n) - 1
    a = []
    hanoi(5, 0, 2, 1, a)
    for k in range(1, len(a) + 1):
        assert a[k - 1][0] == ruler(k)
    assert not legal([0, 0], (1, 0, 1))
    assert not replay(2, [(0, 0, 2)])


tests()
main()
