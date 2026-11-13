class CalcError(Exception):
    pass


def tokenize(src):
    out = []
    num = ""
    for c in src:
        if c.isdigit():
            num += c
            continue
        if num != "":
            out.append(num)
            num = ""
        if c in "+-*/^()":
            out.append(c)
        elif not c.isspace():
            raise CalcError("bad character " + c)
    if num != "":
        out.append(num)
    return out


def prec(op):
    return {"+": 1, "-": 1, "*": 2, "/": 2, "^": 3}.get(op, 0)


def apply(op, a, b):
    if op == "+":
        return a + b
    if op == "-":
        return a - b
    if op == "*":
        return a * b
    if op == "/":
        if b == 0:
            raise CalcError("division by zero")
        q = abs(a) // abs(b)
        return q if (a < 0) == (b < 0) else -q
    if b < 0:
        raise CalcError("negative exponent")
    r = 1
    for _ in range(b):
        r *= a
    return r


class Parser:
    def __init__(self, toks):
        self.toks = toks
        self.pos = 0
        self.rpn = []

    def peek(self):
        if self.pos < len(self.toks):
            return self.toks[self.pos]
        return "end"

    def take(self):
        t = self.peek()
        self.pos += 1
        return t

    def primary(self):
        t = self.take()
        if t == "(":
            v = self.expr(1)
            if self.take() != ")":
                raise CalcError("missing closing parenthesis")
            return v
        if t == "-":
            w = self.expr(3)
            self.rpn.append("neg")
            return 0 - w
        if not t.isdigit():
            raise CalcError("unexpected " + t)
        self.rpn.append(t)
        return int(t)

    def expr(self, lowest):
        lhs = self.primary()
        while prec(self.peek()) >= lowest and prec(self.peek()) > 0:
            op = self.take()
            nxt = prec(op) if op == "^" else prec(op) + 1
            rhs = self.expr(nxt)
            self.rpn.append(op)
            lhs = apply(op, lhs, rhs)
        return lhs


def evaluate(src):
    p = Parser(tokenize(src))
    v = p.expr(1)
    if p.pos != len(p.toks):
        raise CalcError("unexpected " + p.peek())
    return "%d   rpn: %s" % (v, " ".join(p.rpn))


exprs = [
    "1 + 2 * 3",
    "(1 + 2) * 3",
    "2 ^ 3 ^ 2",
    "-2 ^ 2",
    "10 / 3 * 3",
    "7 - 2 - 1",
    "-(4 + 1) * 2",
    "8 / (3 - 3)",
    "2 ^ -1",
    "(1 + 2",
    "4 $ 4",
    "3 +",
    "1 2",
]
for src in exprs:
    try:
        r = evaluate(src)
        print(f"{src:<14} = {r}")
    except CalcError as e:
        print(f"{src:<14} ! {e}")
