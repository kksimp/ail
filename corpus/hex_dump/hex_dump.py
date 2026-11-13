def hex_digits(n, width):
    table = "0123456789abcdef"
    out = ""
    v = n
    k = 0
    while k < width:
        out = table[v % 16] + out
        v = v // 16
        k += 1
    return out


def gutter(chunk):
    out = ""
    for c in chunk:
        if 32 <= c < 127:
            out += chr(c)
        else:
            out += "."
    return out


def dump_line(offset, chunk):
    cells = ""
    for i in range(16):
        cell = hex_digits(chunk[i], 2) if i < len(chunk) else "  "
        sep = "  " if i == 8 else " "
        cells += sep + cell
    return hex_digits(offset, 8) + cells + "  |" + gutter(chunk) + "|"


def dump(text):
    codes = [ord(c) for c in text]
    lines = []
    i = 0
    while i < len(codes):
        end = min(i + 16, len(codes))
        lines.append(dump_line(i, codes[i:end]))
        i += 16
    lines.append(hex_digits(len(codes), 8))
    return lines


def from_hex(spaced):
    out = ""
    for w in spaced.split():
        out += chr(int(w, 16))
    return out


def main():
    sample = "Hello, AIL!\tTabs and\nnewlines show as dots.\nThe end."
    for line in dump(sample):
        print(line)
    print("---")
    for line in dump("short"):
        print(line)
    print(from_hex("48 65 6c 6c 6f 20 41 49 4c"))


main()
