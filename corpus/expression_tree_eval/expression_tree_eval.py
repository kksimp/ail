class DivZero(Exception):
    pass


def from_postfix(text):
    stack = []
    for tok in text.split(" "):
        if tok in ("+", "-", "*", "/"):
            b = stack.pop() if stack else ("num", 0)
            a = stack.pop() if stack else ("num", 0)
            stack.append(({"+": "add", "-": "sub", "*": "mul", "/": "div"}[tok], a, b))
        elif tok == "neg":
            v = stack.pop() if stack else ("num", 0)
            stack.append(("neg", v))
        else:
            stack.append(("num", int(tok)))
    return stack.pop() if stack else ("num", 0)


SYM = {"add": "+", "sub": "-", "mul": "*", "div": "/"}


def show(e):
    if e[0] == "num":
        return str(e[1])
    if e[0] == "neg":
        return "-" + show(e[1])
    return f"({show(e[1])} {SYM[e[0]]} {show(e[2])})"


def depth(e):
    if e[0] == "num":
        return 1
    if e[0] == "neg":
        return 1 + depth(e[1])
    return 1 + max(depth(e[1]), depth(e[2]))


def evaluate(e):
    k = e[0]
    if k == "num":
        return e[1]
    if k == "neg":
        return -evaluate(e[1])
    if k == "add":
        return evaluate(e[1]) + evaluate(e[2])
    if k == "sub":
        return evaluate(e[1]) - evaluate(e[2])
    if k == "mul":
        return evaluate(e[1]) * evaluate(e[2])
    d = evaluate(e[2])
    if d == 0:
        raise DivZero()
    x = evaluate(e[1])
    q = abs(x) // abs(d)
    return q if (x < 0) == (d < 0) else -q


def fold(e):
    if e[0] == "num":
        return e
    if e[0] == "neg":
        f = fold(e[1])
        if f[0] == "num":
            return ("num", -f[1])
        return ("neg", f)
    a = fold(e[1])
    b = fold(e[2])
    rebuilt = (e[0], a, b)
    if a[0] == "num" and b[0] == "num":
        try:
            return ("num", evaluate(rebuilt))
        except DivZero:
            return rebuilt
    return rebuilt


for src in ["3 4 + 2 *", "10 2 3 * - neg", "8 2 4 + /", "1 2 + 5 5 - /", "9 3 /"]:
    tree = from_postfix(src)
    try:
        result = evaluate(tree)
    except DivZero:
        print(f"{show(tree)} fails: division by zero")
        continue
    print(f"{show(tree)} = {result} (depth {depth(tree)}, folded {show(fold(tree))})")
