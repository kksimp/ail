"""A stack calculator driven by an embedded REPL script."""


class CalcError(Exception):
    pass


def read_cmd(line):
    w = line.split()
    head = w[0]
    try:
        return ("push", int(head))
    except ValueError:
        pass
    if head == "push" and len(w) == 2:
        try:
            return ("push", int(w[1]))
        except ValueError:
            raise CalcError("bad number '%s'" % w[1])
    if len(w) == 1:
        if head in ["add", "sub", "mul", "div", "mod"]:
            return ("binary", head)
        if head == "dup":
            return ("dup",)
        if head == "swap":
            return ("swap",)
        if head == "dump":
            return ("dump",)
    if head == "store" and len(w) == 2:
        return ("store", w[1])
    if head == "load" and len(w) == 2:
        return ("load", w[1])
    raise CalcError("unknown command '%s'" % line)


def tdiv(a, b):
    q = abs(a) // abs(b)
    return q if (a < 0) == (b < 0) else -q


class Calc:
    def __init__(self):
        self.stack = []
        self.vars = {}

    def need(self, op, count):
        if len(self.stack) < count:
            raise CalcError("%s needs %d values, stack has %d" % (op, count, len(self.stack)))

    def binary(self, op):
        self.need(op, 2)
        b = self.stack[-1]
        a = self.stack[-2]
        if op in ("div", "mod") and b == 0:
            raise CalcError("%s by zero" % op)
        if op == "add":
            r = a + b
        elif op == "sub":
            r = a - b
        elif op == "mul":
            r = a * b
        elif op == "div":
            r = tdiv(a, b)
        else:
            r = a - tdiv(a, b) * b
        self.stack.pop()
        self.stack.pop()
        self.stack.append(r)
        return ""

    def dump(self):
        if not self.stack:
            return "stack: (empty)"
        return "stack: " + " ".join(str(x) for x in self.stack)

    def exec(self, cmd):
        kind = cmd[0]
        if kind == "push":
            self.stack.append(cmd[1])
            return ""
        if kind == "binary":
            return self.binary(cmd[1])
        if kind == "dup":
            self.need("dup", 1)
            self.stack.append(self.stack[-1])
            return ""
        if kind == "swap":
            self.need("swap", 2)
            x = self.stack.pop()
            y = self.stack.pop()
            self.stack.append(x)
            self.stack.append(y)
            return ""
        if kind == "store":
            if not self.stack:
                raise CalcError("store needs 1 values, stack has 0")
            self.vars[cmd[1]] = self.stack.pop()
            return ""
        if kind == "load":
            if cmd[1] not in self.vars:
                raise CalcError("no variable %s" % cmd[1])
            self.stack.append(self.vars[cmd[1]])
            return ""
        return self.dump()


def main():
    script = ["# ring area: (10 + 4) squared", "push 10", "push 4", "add", "dup", "mul", "dump", "store sq",
              "load sq", "push 3", "div", "dump", "div", "push 0", "div", "load nope", "bogus 1", "push x1",
              "5", "7", "swap", "sub", "dump", "", "-7", "2", "div", "dump", "17", "5", "mod", "dump"]
    calc = Calc()
    failed = 0
    for i, line in enumerate(script):
        text = line.strip()
        if text == "" or text.startswith("#"):
            continue
        print("%2d> %s" % (i + 1, text))
        try:
            out = calc.exec(read_cmd(text))
        except CalcError as err:
            print("    error: %s" % err)
            failed += 1
            continue
        if out:
            print("    " + out)
    print(calc.dump())
    for name in sorted(calc.vars):
        print("%s = %d" % (name, calc.vars[name]))
    print("%d errors" % failed)


main()
