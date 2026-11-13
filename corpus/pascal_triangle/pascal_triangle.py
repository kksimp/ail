def next_row(row):
    out = [1]
    for i in range(1, len(row)):
        out.append(row[i - 1] + row[i])
    out.append(1)
    return out


def triangle(count):
    rows = [[1]]
    for k in range(1, count):
        rows.append(next_row(rows[k - 1]))
    return rows


def choose(n, k):
    row = triangle(n + 1)[n]
    return row[k]


def show_row(row, indent):
    text = " " * indent
    for v in row:
        text = f"{text}{v:>4}"
    return text


def sierpinski(row, indent):
    pad = " " * indent
    marks = []
    for v in row:
        marks.append("*" if v % 2 == 1 else ".")
    return pad + " ".join(marks)


def odd_count(row):
    n = 0
    for v in row:
        if v % 2 == 1:
            n += 1
    return n


def main():
    rows = triangle(10)
    for r in range(len(rows)):
        print(show_row(rows[r], 2 * (len(rows) - 1 - r)))
    sums = [sum(part) for part in rows]
    print("row sums: " + " ".join(str(x) for x in sums))
    print(f"C(20, 10) = {choose(20, 10)}")
    print(f"C(30, 3) = {choose(30, 3)}")
    big = triangle(16)
    for q in range(len(big)):
        print(sierpinski(big[q], len(big) - 1 - q))
    odd_counts = [odd_count(line) for line in big]
    print("odd entries per row: " + " ".join(str(x) for x in odd_counts))


main()
