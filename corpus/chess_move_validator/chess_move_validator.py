"""Chess move validator."""


class MoveError(Exception):
    pass


def is_upper(p):
    return p != p.lower()


class Game:
    def __init__(self):
        fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        cs = []
        for rank in fen.split("/"):
            for ch in rank:
                if ch.isdigit():
                    cs.extend(["."] * int(ch))
                else:
                    cs.append(ch)
        self.cells = cs
        self.white = True
        self.taken = []
        self.played = 0

    def try_move(self, text):
        if len(text) != 4:
            raise MoveError("cannot read move '%s'" % text)
        from_sq = text[0:2]
        to_sq = text[2:4]
        a = square(from_sq)
        b = square(to_sq)
        piece = self.cells[a]
        if piece == ".":
            raise MoveError("no piece on %s" % from_sq)
        if is_upper(piece) != self.white:
            raise MoveError("it is not that piece's turn on %s" % from_sq)
        target = self.cells[b]
        if target != "." and is_upper(target) == is_upper(piece):
            raise MoveError("%s holds a friendly piece" % to_sq)
        v = judge(self.cells, piece, a, b)
        if v == "illegal":
            raise MoveError("%s cannot move %s" % (piece, text))
        if v == "obstructed":
            raise MoveError("path blocked for %s" % text)
        placed = piece
        if piece.lower() == "p" and (b // 8 == 0 or b // 8 == 7):
            placed = "Q" if is_upper(piece) else "q"
        self.cells[b] = placed
        self.cells[a] = "."
        self.white = not self.white
        self.played += 1
        if target == ".":
            return "%s %s" % (piece, text)
        self.taken.append(target)
        return "%s %s takes %s" % (piece, text, target)

    def render(self):
        rows = []
        for r in range(8):
            rows.append("%d %s" % (8 - r, "".join(self.cells[r * 8:r * 8 + 8])))
        rows.append("  abcdefgh")
        return rows


def square(text):
    col = "abcdefgh".find(text[0:1])
    if col < 0:
        raise MoveError("cannot read move '%s'" % text)
    try:
        rank = int(text[1:2])
    except ValueError:
        raise MoveError("cannot read move '%s'" % text)
    if rank < 1 or rank > 8:
        raise MoveError("cannot read move '%s'" % text)
    return (8 - rank) * 8 + col


def sign(n):
    return max(-1, min(1, n))


def slide(cells, a, b):
    dr = b // 8 - a // 8
    dc = b % 8 - a % 8
    step = sign(dr) * 8 + sign(dc)
    pos = a + step
    while pos != b:
        if cells[pos] != ".":
            return "obstructed"
        pos += step
    return "legal"


def judge(cells, piece, a, b):
    dr = b // 8 - a // 8
    dc = b % 8 - a % 8
    adr = abs(dr)
    adc = abs(dc)
    kind = piece.lower()
    straight = dr == 0 or dc == 0
    if kind == "n":
        ok = (adr == 1 and adc == 2) or (adr == 2 and adc == 1)
        return "legal" if ok else "illegal"
    if kind == "k":
        return "legal" if max(adr, adc) == 1 else "illegal"
    if kind == "r" and straight:
        return slide(cells, a, b)
    if kind == "b" and adr == adc:
        return slide(cells, a, b)
    if kind == "q" and (straight or adr == adc):
        return slide(cells, a, b)
    if kind == "p":
        fwd = -1 if is_upper(piece) else 1
        home = 6 if is_upper(piece) else 1
        if dc == 0 and dr == fwd:
            return "legal" if cells[b] == "." else "obstructed"
        if dc == 0 and dr == 2 * fwd and a // 8 == home:
            free = cells[a + fwd * 8] == "." and cells[b] == "."
            return "legal" if free else "obstructed"
        if adc == 1 and dr == fwd and cells[b] != ".":
            return "legal"
    return "illegal"


def main():
    moves = [
        "e2e4", "e7e5", "d1d3", "a7a6", "a1a3", "f1c4", "b8c6", "d1h5", "g8f6",
        "h5f7", "e8e7", "e4", "z9e4", "c1c1", "e7e6", "f7e8", "d7d5", "a2a4",
    ]
    g = Game()
    for m in moves:
        try:
            msg = g.try_move(m)
        except MoveError as err:
            print("rejected %s: %s" % (m, err))
            continue
        print("ok %s" % msg)
    for row in g.render():
        print(row)
    print("moves played %d, captured %s" % (g.played, ",".join(g.taken)))


main()
