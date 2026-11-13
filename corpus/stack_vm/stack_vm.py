class VmError(Exception):
    pass


def assemble(lines):
    labels = {}
    body = []
    for raw in lines:
        line = raw.strip()
        if line.endswith(":"):
            labels[line[:-1]] = len(body)
        elif line != "":
            body.append(line)
    ops = []
    for text in body:
        parts = text.split()
        name = parts[0]
        arg = parts[1] if len(parts) > 1 else ""
        try:
            num = int(arg)
            isnum = True
        except ValueError:
            num = 0
            isnum = False
        if name in ("push", "load", "store"):
            if len(parts) != 2 or not isnum:
                raise VmError("cannot assemble '%s'" % text)
            ops.append((name, num))
        elif name in ("jmp", "jz"):
            if arg not in labels:
                raise VmError("unknown label %s" % arg)
            ops.append((name, labels[arg]))
        elif name in ("add", "sub", "mul", "lt"):
            ops.append(("bin", {"add": "+", "sub": "-", "mul": "*", "lt": "<"}[name]))
        elif name in ("print", "halt"):
            ops.append((name, 0))
        else:
            raise VmError("cannot assemble '%s'" % text)
    return ops


def pop_val(st, pc):
    if not st:
        raise VmError("stack underflow at %d" % pc)
    return st.pop()


def binop(sym, a, b):
    if sym == "+":
        return a + b
    if sym == "-":
        return a - b
    if sym == "*":
        return a * b
    return 1 if a < b else 0


def run(prog):
    st = []
    mem = [0] * 8
    out = []
    pc = 0
    steps = 0
    while pc < len(prog):
        steps += 1
        if steps > 5000:
            raise VmError("step limit reached")
        kind, arg = prog[pc]
        pc += 1
        if kind == "push":
            st.append(arg)
        elif kind == "bin":
            b = pop_val(st, pc - 1)
            a = pop_val(st, pc - 1)
            st.append(binop(arg, a, b))
        elif kind == "load":
            st.append(mem[arg])
        elif kind == "store":
            mem[arg] = pop_val(st, pc - 1)
        elif kind == "jmp":
            pc = arg
        elif kind == "jz":
            cond = pop_val(st, pc - 1)
            if cond == 0:
                pc = arg
        elif kind == "print":
            out.append(str(pop_val(st, pc - 1)))
        elif kind == "halt":
            pc = len(prog)
    return " ".join(out)


def execute(lines):
    return run(assemble(lines))


cases = [
    ("factorial", [
        "push 6", "store 0", "push 1", "store 1",
        "loop:", "load 0", "jz done", "load 1", "load 0", "mul", "store 1",
        "load 0", "push 1", "sub", "store 0", "jmp loop",
        "done:", "load 1", "print", "halt",
    ]),
    ("sum10", [
        "push 0", "store 0", "push 1", "store 1",
        "top:", "load 1", "push 11", "lt", "jz end",
        "load 0", "load 1", "add", "store 0",
        "load 1", "push 1", "add", "store 1", "jmp top",
        "end:", "load 0", "print",
    ]),
    ("countdown", [
        "push 5", "store 0",
        "again:", "load 0", "jz fin", "load 0", "print",
        "load 0", "push 1", "sub", "store 0", "jmp again",
        "fin:", "halt",
    ]),
    ("stray_add", ["push 1", "add", "print"]),
    ("bad_label", ["jmp nowhere"]),
    ("bad_op", ["push x"]),
    ("spin", ["top:", "jmp top"]),
]
for name, lines in cases:
    try:
        r = execute(lines)
        print(f"{name:<10} {r}")
    except VmError as e:
        print(f"{name:<10} error: {e}")
