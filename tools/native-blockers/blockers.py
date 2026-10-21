#!/usr/bin/env python3
"""Native blockers (day 3). Measurement only; run run.sh.

For every program in examples/, it runs `ail build` one program at a time (a
build doesn't run the program, so a window program is no problem), then:

  - counts the programs that build natively;
  - groups the programs that don't by the construct of their first rejection
    (the first `error[...]` line the build prints), using RULES below;
  - lists the ffi.native.1 programs (C linking) on their own.

The groups are checked in order; the first match wins. A message that matches
no rule is printed as OTHER with its text, so a new construct shows up there.
"""
import argparse
import collections
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
AIL = os.path.join(ROOT, "ail")
BUILD_TIMEOUT = 120

# (group, owner, pattern on the first error line). Owners are the lanes that
# were named for them on day 3; "" means no owner was named.
RULES = [
    ("ffi: extern C functions and unsafe blocks", "", r"ffi\.native\.1|`unsafe` blocks"),
    ("OS processes: std.proc", "lanes 177 and 232", r"`proc`"),
    ("Channels, spawn, TaskGroup, Mutex", "lane 178", r"Channel|`spawn`|calling `TaskGroup`|calling `Mutex`|parameter `ch`"),
    ("Std modules: hash, compress, regex, date, csv, log, args, jar, net, bytes, tensor, fs.glob, ask", "",
     r"`(hash|compress|regex|date|csv|log|jar|args|net|bytes|tensor|ask|ui)`|fs\.glob|`bytes` literals"),
    ("Map and list methods, map and set comprehensions", "lane 173",
     r"\.(first|last|sum|top|index_of|union|max|keys|remove|get|contains|pop|join)\(\)|map comprehensions|set comprehensions|comprehension whose element|map of|empty `\{\}`"),
    ("Strings and chars: to_int, match on char, char comparisons", "lane 235",
     r"\.to_int\(\)|`match` on `char`|`char\?`|`\.pos` on `str`"),
    ("Enum variants, trait and impl methods, generics", "lane 179",
     r"calling `[A-Z]|`[A-Z][a-zA-Z]+` isn't|trait `|methods of|generic functions"),
    ("Error structs and error enums, `?` on user types", "optionals thread",
     r"error struct|error enum|`\?` on a `(Expr|Token|Stmt|Json|Value|Flow|Pos)`"),
    ("User JSON parsing and types (Json, parse_json, from_json)", "",
     r"`Json` isn't|parse_json|from_json|row_json"),
    ("Parameter, field or result type outside the subset", "",
     r"its type isn't|its result type|field `[a-z_]+`: its type"),
    ("Attributes: @test", "", r"@test"),
    ("Method calls the message doesn't name", "", r"this method call"),
    ("Contracts, `for mut`, `else` without a value", "lane 177 (else forms)",
     r"contracts|`for mut`|this `else`|this expression has no value|`else` block needs"),
]


def group_of(line):
    for group, owner, pattern in RULES:
        if re.search(pattern, line):
            return group, owner
    return "OTHER", ""


def first_error(stderr):
    for line in stderr.splitlines():
        if "error[" in line:
            return line.strip()
    return ""


def main():
    ap = argparse.ArgumentParser(description="Group the examples that `ail build` rejects by their first blocker.")
    ap.add_argument("--only", default="", help="only examples whose file name contains this text")
    args = ap.parse_args()

    programs = sorted(glob.glob(os.path.join(ROOT, "examples", "*.ail")))
    programs = [p for p in programs if args.only in os.path.basename(p)]
    if not programs:
        print("native-blockers: no examples match", file=sys.stderr)
        return 2

    scratch = tempfile.mkdtemp(prefix="ail-native-blockers-")
    built = []
    rejected = []   # (name, group, owner, first error line, error count)
    ffi = []
    try:
        for path in programs:
            name = os.path.splitext(os.path.basename(path))[0]
            rel = os.path.relpath(path, ROOT)
            try:
                proc = subprocess.run(
                    [AIL, "build", rel, "-o", os.path.join(scratch, name)],
                    cwd=ROOT, capture_output=True, text=True, timeout=BUILD_TIMEOUT,
                )
            except subprocess.TimeoutExpired:
                rejected.append((name, "build timed out", "", "", 0))
                continue
            if proc.returncode == 0:
                built.append(name)
                continue
            errors = [l for l in proc.stderr.splitlines() if "error[" in l]
            first = first_error(proc.stderr)
            if not first:
                rejected.append((name, "build failed without a rule ID", "", proc.stderr.strip()[:140], 0))
                continue
            group, owner = group_of(first)
            if group.startswith("ffi:"):
                ffi.append((name, first))
                continue
            rejected.append((name, group, owner, first, len(errors)))
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    total = len(programs)
    groups = collections.defaultdict(list)
    for name, group, owner, first, count in rejected:
        groups[(group, owner)].append(name)
    print("First rejection, by group (examples whose first blocker is in the group)")
    print()
    print(f"{'examples':>8}  {'group':<70} owner")
    for (group, owner), names in sorted(groups.items(), key=lambda kv: (-len(kv[1]), kv[0][0])):
        print(f"{len(names):>8}  {group:<70} {owner or '-'}")
    print()
    print(f"{len(built)} of {total} examples build natively")
    print()
    print("ffi.native.1 (C linking; the program's `ail run` works):")
    for name, first in ffi:
        print(f"  {name}: {first.split(': ', 1)[-1]}")
    if not ffi:
        print("  none")
    other = [r for r in rejected if r[1] == "OTHER"]
    if other:
        print()
        print("OTHER (no rule matched; add one to RULES if the construct recurs):")
        for name, _, _, first, _ in other:
            print(f"  {name}: {first}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
