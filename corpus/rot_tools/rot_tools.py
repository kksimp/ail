def rotate(c, base, k):
    return chr((ord(c) - base + k) % 26 + base)


def rot_alpha(text, key):
    k = (key % 26 + 26) % 26
    out = ""
    for c in text:
        if "a" <= c <= "z":
            out += rotate(c, 97, k)
        elif "A" <= c <= "Z":
            out += rotate(c, 65, k)
        else:
            out += c
    return out


def rot_digits(text, key):
    k = (key % 10 + 10) % 10
    out = ""
    for c in text:
        if "0" <= c <= "9":
            out += chr((ord(c) - 48 + k) % 10 + 48)
        else:
            out += c
    return out


def rot47(text):
    out = ""
    for c in text:
        n = ord(c)
        if 33 <= n <= 126:
            out += chr(33 + (n - 33 + 47) % 94)
        else:
            out += c
    return out


def mirror(c, lo, hi):
    return chr(lo + hi - ord(c))


def atbash(text):
    out = ""
    for c in text:
        if "a" <= c <= "z":
            out += mirror(c, 97, 122)
        elif "A" <= c <= "Z":
            out += mirror(c, 65, 90)
        else:
            out += c
    return out


def rot13(text):
    return rot_digits(rot_alpha(text, 13), 5)


def main():
    msg = "Meet at Gate 7 at 18:30, bring Zed's key!"
    print("plain:    " + msg)
    print("rot13+5:  " + rot13(msg))
    print("rot47:    " + rot47(msg))
    print("atbash:   " + atbash(msg))
    print("rot-3:    " + rot_alpha(msg, -3))
    print("rot13x2:  " + rot13(rot13(msg)))
    print("rot47x2:  " + rot47(rot47(msg)))
    print("atbash x2: " + atbash(atbash(msg)))
    for k in [1, 5, 13, 25]:
        print("shift %2d: %s" % (k, rot_alpha("Hello, World", k)))
    print(rot_digits("2026-11-13", 3))


main()
