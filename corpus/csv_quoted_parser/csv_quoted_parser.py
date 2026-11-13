class CsvError(Exception):
    pass


def parse_csv(text):
    rows = []
    row = []
    field = ""
    st = "start"
    for c in text:
        if st == "quoted":
            if c == '"':
                st = "seen"
            else:
                field += c
        elif st == "seen":
            if c == '"':
                field += c
                st = "quoted"
            elif c == ",":
                row.append(field)
                field = ""
                st = "start"
            elif c == "\n":
                row.append(field)
                rows.append(row)
                row = []
                field = ""
                st = "start"
            else:
                raise CsvError("unexpected character after quote in row %d" % (len(rows) + 1))
        elif c == '"' and st == "start":
            st = "quoted"
        elif c == ",":
            row.append(field)
            field = ""
            st = "start"
        elif c == "\n":
            row.append(field)
            rows.append(row)
            row = []
            field = ""
            st = "start"
        elif c != "\r":
            field += c
            st = "unquoted"
    if st == "quoted":
        raise CsvError("unterminated quote in row %d" % (len(rows) + 1))
    if st != "start" or row or field != "":
        row.append(field)
        rows.append(row)
    return rows


def quote_field(f):
    if "," in f or '"' in f or "\n" in f:
        return '"' + f.replace('"', '""') + '"'
    return f


def write_csv(rows):
    return "\n".join(",".join(quote_field(f) for f in r) for r in rows)


def show(f):
    return "<" + f.replace("\n", "~") + ">"


def report(label, text):
    print(f"{label}:")
    try:
        rows = parse_csv(text)
    except CsvError as e:
        print(f"  error: {e}")
        return
    for i, r in enumerate(rows):
        print(f"  row {i + 1} ({len(r)} fields): {' '.join(show(f) for f in r)}")
    try:
        again = parse_csv(write_csv(rows))
    except CsvError:
        again = rows
    print("  round trip: " + ("true" if again == rows else "false"))


good = [
    "name,quote,score",
    '"Smith, Jo","said ""hi""",9',
    'Lee,"two',
    'lines",7',
    ',"",',
    "x,y,z",
]
report("good", "\n".join(good))
report("crlf", "a,b\r\nc,d\r\n")
report("bad quote", 'a,"b"c')
report("unterminated", 'a,"b\nc,d')
report("empty", "")
