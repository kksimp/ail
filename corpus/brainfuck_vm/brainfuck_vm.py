class BfError(Exception):
    pass


def build_jumps(code):
    jumps = []
    stack = []
    for i in range(len(code)):
        jumps.append(0)
        if code[i] == "[":
            stack.append(i)
        elif code[i] == "]":
            if not stack:
                raise BfError("unmatched bracket at %d" % i)
            j = stack.pop()
            jumps[j] = i
            jumps[i] = j
    if stack:
        raise BfError("unmatched bracket at %d" % stack[-1])
    return jumps


def run(src, data):
    code = list(src)
    jumps = build_jumps(code)
    feed = list(data)
    tape = [0] * 30
    out = []
    ip = 0
    ptr = 0
    fed = 0
    steps = 0
    while ip < len(code):
        steps += 1
        if steps > 20000:
            raise BfError("step limit reached")
        c = code[ip]
        if c == ">":
            ptr += 1
        elif c == "<":
            ptr -= 1
        elif c == "+":
            tape[ptr] = (tape[ptr] + 1) % 256
        elif c == "-":
            tape[ptr] = (tape[ptr] + 255) % 256
        elif c == ".":
            out.append(tape[ptr])
        elif c == ",":
            if fed < len(feed):
                tape[ptr] = ord(feed[fed])
                fed += 1
            else:
                tape[ptr] = 0
        elif c == "[" and tape[ptr] == 0:
            ip = jumps[ip]
        elif c == "]" and tape[ptr] != 0:
            ip = jumps[ip]
        if ptr < 0 or ptr >= len(tape):
            raise BfError("pointer left the tape at %d" % ip)
        ip += 1
    return out


def render(codes):
    s = ""
    for n in codes:
        if 32 <= n < 127:
            s += chr(n)
        else:
            s += "<%d>" % n
    return s


hi = "++++++++[>+++++++++<-]>.+++++++++++++++++++++++++++++++++.>++++++++++."
digits = "++++++++[>>++++++<<-]>++++++++++[>.+<-]"
cases = [
    ("hi", hi, ""),
    ("digits", digits, ""),
    ("cat", ",[.,]", "ail vm"),
    ("reverse", ">,[>,]<[.<]", "stressed"),
    ("wrap", "-.", ""),
    ("unmatched", "[+]]", ""),
    ("runaway", "+[]", ""),
    ("falloff", "<", ""),
]
for name, src, data in cases:
    try:
        out = run(src, data)
        print(f"{name:<9} {render(out)}")
    except BfError as e:
        print(f"{name:<9} error: {e}")
