import os, tempfile, shutil


def data():
    return [
        ("north", [120, 135, 150, 180]),
        ("south", [90, 85, 110, 95]),
        ("east", [200, 210, 190, 250]),
        ("west", [60, 75, 80, 70]),
    ]


def format_row(r, grand):
    region, q = r
    total = sum(q)
    share = total * 100 // grand
    cells = [f"{x:>6}" for x in q]
    return f"{region:<8}{''.join(cells)}{total:>7}{share:>5}%"


def report_lines(rows):
    grand = sum(sum(q) for _, q in rows)
    rule = "-" * 45
    out = ["SALES REPORT", f"{'region':<8}{'Q1':>6}{'Q2':>6}{'Q3':>6}{'Q4':>6}{'total':>7}{'share':>6}", rule]
    for r in rows:
        out.append(format_row(r, grand))
    out.append(rule)
    sums = [0, 0, 0, 0]
    for _, q in rows:
        for i in range(4):
            sums[i] += q[i]
    cells = [f"{x:>6}" for x in sums]
    out.append(f"{'all':<8}{''.join(cells)}{grand:>7}{100:>5}%")
    return out


def main():
    d = tempfile.mkdtemp(prefix="corpus_text_report_writer")
    p = os.path.join(d, "report.txt")
    with open(p, "w", newline="") as f:
        for l in report_lines(data()):
            f.write(l + "\n")
    with open(p, newline="") as f:
        text = f.read()
    for n, l in enumerate(text.splitlines(), 1):
        print(f"{n:>2}| {l}")
    print(f"file size: {os.path.getsize(p)} bytes")
    shutil.rmtree(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
