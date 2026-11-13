def rotate_in_place(m, n):
    for layer in range(n // 2):
        lo = layer
        hi = n - 1 - layer
        for i in range(lo, hi):
            off = i - lo
            saved = m[lo * n + i]
            m[lo * n + i] = m[(hi - off) * n + lo]
            m[(hi - off) * n + lo] = m[hi * n + hi - off]
            m[hi * n + hi - off] = m[i * n + hi]
            m[i * n + hi] = saved


def rotate_ccw(data, rows, cols):
    out = []
    for i in range(cols):
        for j in range(rows):
            out.append(data[j * cols + cols - 1 - i])
    return out


def transpose(data, rows, cols):
    out = []
    for i in range(cols):
        for j in range(rows):
            out.append(data[j * cols + i])
    return out


def turn_half(data):
    return list(reversed(data))


def sequence(count):
    return [k + 1 for k in range(count)]


def show(label, data, cols):
    print(f"{label}:")
    for r in range(len(data) // cols):
        cells = [data[r * cols + c] for c in range(cols)]
        line = "".join([f" {v:>2}" for v in cells])
        print(line)


def main():
    sq = sequence(16)
    show("4x4", sq, 4)
    rotate_in_place(sq, 4)
    show("4x4 clockwise", sq, 4)
    rotate_in_place(sq, 4)
    rotate_in_place(sq, 4)
    rotate_in_place(sq, 4)
    print(f"after four turns equals start: {'true' if sq == sequence(16) else 'false'}")
    odd = sequence(9)
    turned = list(odd)
    rotate_in_place(turned, 3)
    show("3x3 clockwise", turned, 3)
    rect = sequence(15)
    show("3x5", rect, 5)
    show("3x5 counter-clockwise", rotate_ccw(rect, 3, 5), 3)
    show("3x5 transposed", transpose(rect, 3, 5), 3)
    print(f"half turn: {' '.join(str(x) for x in turn_half(sequence(6)))}")


main()
