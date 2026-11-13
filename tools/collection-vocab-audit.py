#!/usr/bin/env python3
"""Check the shared collection vocabulary (type.coll.28-32, ADR 1099).

One list of names serves `List`, `Set`, `Map`, `Seq` and `Deque`. This fails
when a name is on one collection and not on a sibling, unless
`tools/collection-vocab-exceptions.txt` has a line `Collection.name reason` for
the gap. An exception whose gap no longer exists fails too, so the file cannot
go stale. (`str.chars()` gives a `List`, so it is covered by `List`.)

Sources: the method name lists of `compiler/check/builtins.ail` (List, Set,
Map) and the `fn Seq.NAME` and `fn Deque.NAME` of `tools/native/prelude.ail`.

Run with `python3 -I tools/collection-vocab-audit.py` from the repository root.
"""
import re
import sys

VOCAB = [
    "map", "filter", "any", "all", "find", "count", "fold", "flat_map", "take",
    "skip", "take_while", "skip_while", "zip", "enumerate", "sum", "min", "max",
    "sorted", "group_by", "contains", "is_empty", "len",
]
COLLECTIONS = ["List", "Set", "Map", "Seq", "Deque"]
# Methods a collection has that are the same meaning under a longer name.
ALIASES = {("Map", "contains"): ["contains"]}


def builtin_lists(path):
    text = open(path, encoding="utf-8").read()
    out = {}
    for kind in ("LIST", "SET", "MAP"):
        m = re.search(r"pub %s_METHODS := \[(.*?)\n\]" % kind, text, re.S)
        if not m:
            sys.exit("%s: no %s_METHODS list" % (path, kind))
        out[kind.capitalize()] = set(re.findall(r'"(\w+)"', m.group(1)))
    return out


def prelude_methods(path):
    text = open(path, encoding="utf-8").read()
    out = {"Seq": set(), "Deque": set()}
    for kind in out:
        out[kind] = set(re.findall(r"^fn %s\.(\w+)" % kind, text, re.M))
    return out


def exceptions(path):
    out = {}
    for n, line in enumerate(open(path, encoding="utf-8"), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(None, 1)
        if len(parts) < 2 or "." not in parts[0]:
            sys.exit("%s:%d: want `Collection.name reason`" % (path, n))
        coll, name = parts[0].split(".", 1)
        if coll not in COLLECTIONS or name not in VOCAB:
            sys.exit("%s:%d: unknown %s" % (path, n, parts[0]))
        out[(coll, name)] = n
    return out


def main():
    have = builtin_lists("compiler/check/builtins.ail")
    have.update(prelude_methods("tools/native/prelude.ail"))
    excepted = exceptions("tools/collection-vocab-exceptions.txt")
    failed = []
    for name in VOCAB:
        owners = [c for c in COLLECTIONS if name in have[c]]
        if not owners:
            failed.append("`%s` is on no collection" % name)
            continue
        for c in COLLECTIONS:
            if name in have[c]:
                if (c, name) in excepted:
                    failed.append(
                        "%s.%s exists, so its exception is stale" % (c, name))
                continue
            if (c, name) not in excepted:
                failed.append(
                    "%s has `%s` (%s) but %s doesn't, and there is no exception line"
                    % (", ".join(owners), name, owners[0], c))
    for (c, name) in excepted:
        if name not in VOCAB:
            failed.append("%s.%s is not in the vocabulary" % (c, name))
    if failed:
        print("collection vocabulary gaps:")
        for f in failed:
            print("  " + f)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
