def numbered(rows, cols):
    return [k + 1 for k in range(rows * cols)]


def spiral_order(data, rows, cols):
    out = []
    top = 0
    bottom = rows - 1
    left = 0
    right = cols - 1
    while top <= bottom and left <= right:
        for c in range(left, right + 1):
            out.append(data[top * cols + c])
        top += 1
        if top <= bottom:
            for r in range(top, bottom + 1):
                out.append(data[r * cols + right])
        right -= 1
        if top <= bottom and left <= right:
            for j in range(0, right - left + 1):
                out.append(data[bottom * cols + right - j])
            bottom -= 1
        if left <= right and top <= bottom:
            for k in range(0, bottom - top + 1):
                out.append(data[(bottom - k) * cols + left])
            left += 1
    return out


def spiral_fill(rows, cols):
    data = [0] * (rows * cols)
    dxs = [1, 0, -1, 0]
    dys = [0, 1, 0, -1]
    x = 0
    y = 0
    d = 0
    for n in range(1, rows * cols + 1):
        data[y * cols + x] = n
        nx = x + dxs[d]
        ny = y + dys[d]
        if nx < 0 or nx >= cols or ny < 0 or ny >= rows or data[ny * cols + nx] != 0:
            d = (d + 1) % 4
        x += dxs[d]
        y += dys[d]
    return data


def rotate_cw(data, rows, cols):
    out = []
    for i in range(cols):
        for j in range(rows):
            out.append(data[(rows - 1 - j) * cols + i])
    return out


def show(label, data, cols):
    print(f"{label}:")
    for r in range(len(data) // cols):
        line = ""
        for c in range(cols):
            v = data[r * cols + c]
            line = f"{line}{v:>4}"
        print(line)


def main():
    grid = numbered(3, 4)
    show("3x4 matrix", grid, 4)
    print("spiral order: " + " ".join(str(x) for x in spiral_order(grid, 3, 4)))
    filled = spiral_fill(4, 5)
    show("4x5 spiral fill", filled, 5)
    show("rotated clockwise", rotate_cw(grid, 3, 4), 3)
    single = numbered(4, 1)
    print("one column: " + " ".join(str(x) for x in spiral_order(single, 4, 1)))
    wide = numbered(1, 5)
    print("one row: " + " ".join(str(x) for x in spiral_order(wide, 1, 5)))
    big = spiral_fill(6, 6)
    diagonal = [big[k * 6 + k] for k in range(6)]
    print("6x6 diagonal: " + " ".join(str(x) for x in diagonal))


main()
