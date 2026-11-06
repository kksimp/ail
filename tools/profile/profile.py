#!/usr/bin/env python3
"""ail profile (ADR 0948): turn the samples a program wrote with AIL_PROFILE=1 into JSON.

    profile.py EXE SAMPLES [--top N]

SAMPLES holds the program's standard error: `ailprof 1 samples=N dropped=D hz=H`
and `S ADDR ADDR ...` lines (decimal code addresses, innermost first). Addresses
are resolved with addr2line over the DWARF of EXE (ADR 0947). Frames after the
first are return addresses, so one byte is taken off before the lookup.
"""
import json, subprocess, sys

def main():
    args = sys.argv[1:]
    top = 20
    if "--top" in args:
        i = args.index("--top")
        top = int(args[i + 1])
        del args[i:i + 2]
    exe, samples_path = args
    hdr = {}
    stacks = []
    for line in open(samples_path, errors="replace"):
        if line.startswith("ailprof 1 "):
            hdr = dict(kv.split("=") for kv in line.split()[2:])
        elif line.startswith("S "):
            stacks.append([int(x) for x in line.split()[1:]])
    keys = sorted({a - (0 if k == 0 else 1) for st in stacks for k, a in enumerate(st)})
    where = {}
    if keys:
        out = subprocess.run(["addr2line", "-f", "-C", "-e", exe], input="\n".join(hex(a) for a in keys) + "\n",
                             capture_output=True, text=True).stdout.split("\n")
        for k, a in enumerate(keys):
            fn, loc = out[2 * k], out[2 * k + 1]
            where[a] = (fn, loc if not loc.startswith("??") else "")
    selfc, incl, lines, callers = {}, {}, {}, {}
    for st in stacks:
        frames = [(where.get(a - (0 if k == 0 else 1), ("??", ""))) for k, a in enumerate(st)]
        fn0, loc0 = frames[0]
        selfc[fn0] = selfc.get(fn0, 0) + 1
        if loc0:
            lines[loc0] = lines.get(loc0, 0) + 1
        for fn in {f for f, _ in frames}:
            incl[fn] = incl.get(fn, 0) + 1
        if len(frames) > 1:
            pair = (fn0, frames[1][0])
            callers[pair] = callers.get(pair, 0) + 1
    n = len(stacks) or 1
    rows = sorted(incl, key=lambda f: (-selfc.get(f, 0), -incl[f], f))[:top]
    print(json.dumps({
        "samples": len(stacks), "hz": int(hdr.get("hz", 0)), "dropped": int(hdr.get("dropped", 0)),
        "functions": [{"name": f, "self": selfc.get(f, 0), "self_pct": round(100 * selfc.get(f, 0) / n, 1),
                       "total": incl[f], "total_pct": round(100 * incl[f] / n, 1)} for f in rows],
        "lines": [{"at": l, "samples": c} for l, c in sorted(lines.items(), key=lambda kv: -kv[1])[:top]],
        "calls": [{"from": a, "to": b, "samples": c} for (a, b), c in sorted(callers.items(), key=lambda kv: -kv[1])[:top]],
        "note": "lines are exact at calls and checked operations (ADR 0947); other code reports the nearest earlier row",
    }, indent=1))

main()
