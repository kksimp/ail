import base64
import binascii


def encode(text):
    return base64.b64encode(text.encode("ascii")).decode("ascii")


def decode(text):
    stripped = text.replace("=", "")
    if len(stripped) % 4 == 1:
        return None
    for c in stripped:
        if c not in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/":
            return None
    padded = stripped + "=" * (-len(stripped) % 4)
    try:
        return base64.b64decode(padded, validate=True).decode("ascii")
    except (binascii.Error, UnicodeDecodeError):
        return None


def main():
    for s in ["Man", "Ma", "M", "", "Hello, World!", "AIL is fun", "foobar", "~~~???"]:
        coded = encode(s)
        back = decode(coded)
        if back is None:
            back = "<invalid>"
        print("%-14s %-22s %s" % (s, coded, "true" if back == s else "false"))
    for bad in ["@@@@", "TWF", "T", "TW Fu"]:
        r = decode(bad)
        if r is None:
            print("decode %s: invalid" % bad)
        else:
            print("decode %s: ok" % bad)


main()
