def tokenize(src):
    out = []
    num = ""
    for c in src:
        if "0" <= c <= "9":
            num += c
            continue
        if num != "":
            out.append(num)
            num = ""
        if not c.isspace():
            out.append(c)
    if num != "":
        out.append(num)
    return out


class Parser:
    def __init__(self, toks):
        self.toks = toks
        self.pos = 0
        self.bad = False

    def peek(self):
        if self.pos < len(self.toks):
            return self.toks[self.pos]
        return ""

    def take(self):
        t = self.peek()
        self.pos += 1
        return t

    def expr(self):
        v = self.term()
        while self.peek() == "+" or self.peek() == "-":
            op = self.take()
            r = self.term()
            if op == "+":
                v += r
            else:
                v -= r
        return v

    def term(self):
        v = self.factor()
        while self.peek() == "*" or self.peek() == "/":
            op = self.take()
            r = self.factor()
            if op == "*":
                v *= r
            elif r == 0:
                self.bad = True
            else:
                q = abs(v) // abs(r)
                v = q if (v < 0) == (r < 0) else -q
        return v

    def factor(self):
        t = self.take()
        if t == "(":
            v = self.expr()
            if self.take() != ")":
                self.bad = True
            return v
        if t == "-":
            return 0 - self.factor()
        if t.isdigit() and t.isascii():
            return int(t)
        self.bad = True
        return 0


def evaluate(src):
    p = Parser(tokenize(src))
    v = p.expr()
    if p.bad or p.pos != len(p.toks):
        return None
    return v


def show(src):
    v = evaluate(src)
    if v is None:
        return f"{src} => error"
    return f"{src} => {v}"


def main():
    for src in ["1 + 2 * 3", "(1 + 2) * 3", "-7 / 2", "2 * (3 + 4) - 5", "10 / (5 - 5)", "2 +", "8 / 2 / 2", "3 $ 4"]:
        print(show(src))


main()
