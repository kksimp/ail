def tokenize(src):
    out = []
    cur = ""
    for c in src:
        if c.isdigit() or c.isalpha():
            cur += c
            continue
        if cur != "":
            out.append(cur)
            cur = ""
        if not c.isspace():
            out.append(c)
    if cur != "":
        out.append(cur)
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
            v = ("bin", op, v, r)
        return v

    def term(self):
        v = self.factor()
        while self.peek() == "*" or self.peek() == "/":
            op = self.take()
            r = self.factor()
            v = ("bin", op, v, r)
        return v

    def factor(self):
        t = self.take()
        if t == "(":
            v = self.expr()
            if self.take() != ")":
                self.bad = True
            return v
        if t == "-":
            return ("neg", self.factor())
        first = t[0] if t else " "
        if first.isdigit():
            try:
                return ("num", int(t))
            except ValueError:
                self.bad = True
                return ("num", 0)
        if first.isalpha():
            return ("var", t)
        self.bad = True
        return ("num", 0)


def parse(src):
    p = Parser(tokenize(src))
    e = p.expr()
    if p.bad or p.pos != len(p.toks):
        return None
    return e


def prec(op):
    if op == "+" or op == "-":
        return 1
    return 2


def prec_of(e):
    if e[0] == "bin":
        return prec(e[1])
    return 9


def render(e):
    if e[0] == "num":
        return str(e[1])
    if e[0] == "var":
        return e[1]
    if e[0] == "neg":
        if prec_of(e[1]) < 9:
            return f"-({render(e[1])})"
        return f"-{render(e[1])}"
    _, op, l, r = e
    ls = f"({render(l)})" if prec_of(l) < prec(op) else render(l)
    rs = f"({render(r)})" if prec_of(r) <= prec(op) else render(r)
    return f"{ls} {op} {rs}"


def full(e):
    if e[0] == "num":
        return str(e[1])
    if e[0] == "var":
        return e[1]
    if e[0] == "neg":
        return f"-{full(e[1])}"
    _, op, l, r = e
    return f"({full(l)} {op} {full(r)})"


def show(src):
    e = parse(src)
    if e is None:
        print(f"{src}: syntax error")
        return
    text = render(e)
    again = parse(text)
    if again is None:
        again = ("num", -1)
    stable = again == e
    print(src)
    print(f"  normal:  {text}")
    print(f"  full:    {full(e)}")
    print(f"  re-parse stable: {'true' if stable else 'false'}")


def main():
    for src in [
        "1+2*3",
        "(1+2)*3",
        "a - (b - c)",
        "((a*b))/(c*d)",
        "-(x+1)*-y",
        "1 + * 2",
        "(1+2",
    ]:
        show(src)


main()
