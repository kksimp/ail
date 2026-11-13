def kind_name(k):
    return ["copycat", "counter", "cycler", "rocky", "wild"][k]


def beats(a, b):
    return (a - b + 3) % 3 == 1


class Player:
    def __init__(self, k):
        self.kind, self.last, self.turn, self.seed = k, -1, 0, 7

    def choose(self):
        if self.kind == 0:
            m = 0 if self.last < 0 else self.last
        elif self.kind == 1:
            m = 1 if self.last < 0 else (self.last + 1) % 3
        elif self.kind == 2:
            m = self.turn % 3
        elif self.kind == 3:
            m = 0
        else:
            self.seed = (self.seed * 1103515245 + 12345) % 2147483648
            m = (self.seed // 65536) % 3
        self.turn += 1
        return m

    def see(self, opp):
        self.last = opp


def play_match(ka, kb, rounds):
    pa, pb = Player(ka), Player(kb)
    wa = wb = 0
    for r in range(rounds):
        ma = pa.choose()
        mb = pb.choose()
        if beats(ma, mb):
            wa += 1
        elif beats(mb, ma):
            wb += 1
        pa.see(mb)
        pb.see(ma)
    return (wa, wb)


def main():
    points = [0] * 5
    for i in range(5):
        for j in range(i + 1, 5):
            a, b = play_match(i, j, 50)
            print(f"{kind_name(i)} vs {kind_name(j)}: {a}-{b}")
            if a > b:
                points[i] += 3
            elif b > a:
                points[j] += 3
            else:
                points[i] += 1
                points[j] += 1
    rows = [(kind_name(k), points[k]) for k in range(5)]
    ranked = sorted(rows, key=lambda p: -p[1])
    print("standings")
    for name, pts in ranked:
        print(f"  {name:<8}{pts:>3}")


def tests():
    assert beats(1, 0) and beats(2, 1) and beats(0, 2) and not beats(0, 0)
    a, b = play_match(3, 1, 10)
    assert a == 0 and b >= 9
    assert play_match(2, 2, 12) == (0, 0)


tests()
main()
