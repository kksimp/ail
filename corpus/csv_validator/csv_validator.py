def is_int(s):
    start = 0
    if len(s) > 0 and s[0] == "-":
        start = 1
    if len(s) <= start:
        return False
    for c in s[start:]:
        if not ("0" <= c <= "9"):
            return False
    return True


def all_digits(s):
    if s == "":
        return False
    for c in s:
        if not ("0" <= c <= "9"):
            return False
    return True


def days_in_month(y, m):
    if m == 2:
        leap = y % 4 == 0 and (y % 100 != 0 or y % 400 == 0)
        return 29 if leap else 28
    if m in (4, 6, 9, 11):
        return 30
    return 31


def valid_date(s):
    parts = s.split("-")
    if len(parts) != 3:
        return False
    if len(parts[0]) != 4 or len(parts[1]) != 2 or len(parts[2]) != 2:
        return False
    if not (all_digits(parts[0]) and all_digits(parts[1]) and all_digits(parts[2])):
        return False
    y = int(parts[0])
    m = int(parts[1])
    d = int(parts[2])
    return 1 <= m <= 12 and 1 <= d <= days_in_month(y, m)


def valid_email(s):
    parts = s.split("@")
    if len(parts) != 2 or parts[0] == "" or parts[1] == "":
        return False
    domain = parts[1]
    dot = domain.find(".")
    if dot < 0:
        return False
    return dot > 0 and not domain.endswith(".")


def check_cell(kind, v):
    if kind == "int":
        return "" if is_int(v) else "not an integer"
    if kind == "email":
        return "" if valid_email(v) else "bad e-mail address"
    if kind == "date":
        return "" if valid_date(v) else "bad date"
    return "empty" if v.strip() == "" else ""


def validate(text):
    lines = text.split("\n")
    problems = []
    names = []
    kinds = []
    for col in lines[0].split(","):
        pair = col.split(":")
        names.append(pair[0])
        kinds.append(pair[1])
    seen = set()
    for i in range(1, len(lines)):
        cells = lines[i].split(",")
        line_no = i + 1
        if len(cells) != len(names):
            problems.append(f"line {line_no}: expected {len(names)} fields, found {len(cells)}")
            continue
        for j in range(len(cells)):
            msg = check_cell(kinds[j], cells[j])
            if msg != "":
                problems.append(f"line {line_no}: {names[j]} '{cells[j]}': {msg}")
        if names[0] == "id":
            fresh = cells[0] not in seen
            seen.add(cells[0])
            if not fresh:
                problems.append(f"line {line_no}: duplicate id {cells[0]}")
    return problems


def main():
    text = "\n".join([
        "id:int,name:text,email:email,joined:date",
        "1,Ann,ann@example.com,2024-02-29",
        "2,Bob,bob@example,2023-13-01",
        "3,,cy@mail.org,2023-02-29",
        "x4,Dee,dee@site.io,2022-12-31",
        "5,Eve,eve@site.io",
        "1,Fay,fay@site.io,2021-04-31",
        "6,Gus,@site.io,2020-06-30",
    ])
    problems = validate(text)
    for p in problems:
        print(p)
    rows = len(text.split("\n")) - 1
    print(f"{rows} rows checked, {len(problems)} problems")


main()
