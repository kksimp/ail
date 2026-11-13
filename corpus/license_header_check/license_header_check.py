WANT_YEAR = 2026


def comment_marker(name):
    if name.endswith(".py") or name.endswith(".sh"):
        return "#"
    if name.endswith(".sql"):
        return "--"
    return "//"


def check(f):
    name, text = f
    lines = text.splitlines()
    if lines and lines[0].startswith("#!"):
        lines = lines[1:]
    mark = comment_marker(name)
    spdx = f"{mark} SPDX-License-Identifier:"
    if len(lines) < 2 or not lines[0].startswith(spdx):
        return ("missing", "no SPDX line")
    ident = lines[0][len(spdx):].strip()
    if ident != "Apache-2.0":
        return ("license", ident)
    words = lines[1].split()
    if len(words) < 5 or words[0] != mark or words[1] != "Copyright" or words[2] != "(c)":
        return ("copyright", "malformed line 2")
    try:
        year = int(words[3])
    except ValueError:
        return ("copyright", f"bad year {words[3]}")
    if year != WANT_YEAR:
        return ("year", f"{year}")
    return ("ok", "")


def sample():
    return [
        ("src/a.go", "// SPDX-License-Identifier: Apache-2.0\n// Copyright (c) 2026 Acme\npackage a"),
        ("tools/run.sh", "#!/bin/sh\n# SPDX-License-Identifier: Apache-2.0\n# Copyright (c) 2026 Acme\necho hi"),
        ("tools/old.py", "# SPDX-License-Identifier: Apache-2.0\n# Copyright (c) 2024 Acme\nx = 1"),
        ("src/b.c", "int main(void) { return 0; }"),
        ("db/init.sql", "-- SPDX-License-Identifier: GPL-3.0-only\n-- Copyright (c) 2026 Acme\nselect 1;"),
        ("src/c.c", "// SPDX-License-Identifier: Apache-2.0\n// copyright 2026 Acme\nint x;"),
        ("src/d.c", "// SPDX-License-Identifier: Apache-2.0\n// Copyright (c) twenty Acme"),
        ("empty.py", ""),
    ]


def main():
    files = sample()
    for f in files:
        kind, detail = check(f)
        extra = "" if detail == "" else f" ({detail})"
        print(f"{f[0]:<14} {kind}{extra}")
    print("summary:")
    for kind in ["ok", "missing", "license", "copyright", "year"]:
        n = 0
        for f in files:
            if check(f)[0] == kind:
                n += 1
        print(f"  {kind:<10} {n}")


main()
