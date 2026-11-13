def clean(s):
    out = ""
    for c in s:
        if c != "-" and c != " ":
            out += c
    return out


def values(s):
    out = []
    for i in range(len(s)):
        c = s[i]
        if c == "X" and i == 9 and len(s) == 10:
            out.append(10)
        else:
            if not ("0" <= c <= "9"):
                return None
            out.append(int(c))
    return out


def sum10(vs):
    total = 0
    for i in range(len(vs)):
        total += vs[i] * (10 - i)
    return total


def sum13(vs):
    total = 0
    for i in range(len(vs)):
        if i % 2 == 0:
            total += vs[i]
        else:
            total += vs[i] * 3
    return total


def is_valid_10(s):
    vs = values(clean(s))
    if vs is None:
        return False
    return len(vs) == 10 and sum10(vs) % 11 == 0


def is_valid_13(s):
    vs = values(clean(s))
    if vs is None:
        return False
    return len(vs) == 13 and sum13(vs) % 10 == 0


def to_13(s):
    vs = values(clean(s))
    if vs is None:
        return ""
    if len(vs) != 10:
        return ""
    body = [9, 7, 8] + vs[0:9]
    chk = (10 - sum13(body) % 10) % 10
    return "".join(str(d) for d in body) + str(chk)


def to_10(s):
    vs = values(clean(s))
    if vs is None:
        return ""
    if len(vs) != 13 or vs[0] != 9 or vs[1] != 7 or vs[2] != 8:
        return ""
    core = vs[3:12]
    chk = (11 - sum10(core) % 11) % 11
    mark = "X" if chk == 10 else str(chk)
    return "".join(str(d) for d in core) + mark


def kind(s):
    if is_valid_10(s):
        return "ISBN-10"
    if is_valid_13(s):
        return "ISBN-13"
    return "invalid"


def main():
    for s in ["0-306-40615-2", "978-0-306-40615-7", "0-8044-2957-X", "0-306-40615-3", "9780306406158", "080442957X", "12345", "0-306-4O615-2"]:
        print("%-20s %s" % (s, kind(s)))
    for s in ["0-306-40615-2", "0-8044-2957-X", "123"]:
        print("%s -> %s" % (s, to_13(s)))
    for s in ["978-0-306-40615-7", "9780804429573", "9791234567896"]:
        print("%s -> %s" % (s, to_10(s)))


main()
