class RpnError(Exception):
    pass


def is_int(tok):
    body = tok[1:] if tok.startswith("-") else tok
    return body.isdigit()


def eval_rpn(expr):
    st = []
    for tok in expr.split():
        if is_int(tok):
            st.append(int(tok))
            continue
        if tok == "dup":
            if not st:
                raise RpnError(f"stack underflow at {tok}")
            st.append(st[-1])
            continue
        if tok == "drop":
            if not st:
                raise RpnError(f"stack underflow at {tok}")
            st.pop()
            continue
        if tok == "swap":
            if len(st) < 2:
                raise RpnError(f"stack underflow at {tok}")
            x = st.pop()
            y = st.pop()
            st.append(x)
            st.append(y)
            continue
        if tok in ["+", "-", "*", "/"]:
            if len(st) < 2:
                raise RpnError(f"stack underflow at {tok}")
            b = st.pop()
            a = st.pop()
            if tok == "+":
                r = a + b
            elif tok == "-":
                r = a - b
            elif tok == "*":
                r = a * b
            else:
                if b == 0:
                    raise RpnError("division by zero")
                r = abs(a) // abs(b)
                if (a < 0) != (b < 0):
                    r = -r
            st.append(r)
            continue
        raise RpnError(f"unknown token {tok}")
    if not st:
        raise RpnError("stack underflow at end")
    if len(st) > 1:
        raise RpnError(f"{len(st)} values left on the stack")
    return st[0]


def main():
    programs = [
        "3 4 +",
        "5 1 2 + 4 * + 3 -",
        "2 dup * dup *",
        "10 3 swap /",
        "7 -2 *",
        "1 +",
        "8 0 /",
        "4 5 pow",
        "1 2 3",
        "",
        "9 drop",
    ]
    ok = 0
    for p in programs:
        try:
            v = eval_rpn(p)
            print(f"{p} => {v}")
            ok += 1
        except RpnError as err:
            print(f"{p} => error: {err}")
    print(f"{ok} of {len(programs)} programs succeeded")


main()
