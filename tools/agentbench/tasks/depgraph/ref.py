#!/usr/bin/env python3
"""Reference oracle for the depgraph task (same contract as spec.md)."""
import re, sys
N = r"[A-Za-z0-9_.-]+"

def main(argv):
    ok = (len(argv) == 1 and argv[0] in ("order", "cycles")) or \
         (len(argv) == 2 and argv[0] in ("deps", "rdeps") and re.fullmatch(N, argv[1]))
    if not ok:
        sys.stderr.write("usage: depgraph order|cycles|deps NAME|rdeps NAME\n"); return 1
    nodes, edges, bad = set(), set(), 0
    for k, ln in enumerate(sys.stdin.buffer.read().decode("utf-8", "replace").split("\n"), 1):
        if ln == "" or ln.startswith("#"): continue
        m = re.fullmatch("(%s) -> (%s)" % (N, N), ln)
        if m: nodes |= {m[1], m[2]}; edges.add((m[1], m[2])); continue
        if re.fullmatch(N, ln): nodes.add(ln); continue
        sys.stderr.write("error: line %d: bad line\n" % k); bad += 1
    needs = {n: set() for n in nodes}; needed = {n: set() for n in nodes}
    for a, b in edges: needs[a].add(b); needed[b].add(a)
    def reach(adj, s):
        seen, st = set(), list(adj[s])
        while st:
            x = st.pop()
            if x not in seen: seen.add(x); st.extend(adj[x])
        return seen
    key = lambda s: s.encode()
    cmd = argv[0]
    if cmd in ("deps", "rdeps"):
        if argv[1] not in nodes:
            sys.stderr.write("error: unknown %s\n" % argv[1]); return 4
        out = sorted(reach(needs if cmd == "deps" else needed, argv[1]), key=key)
    elif cmd == "cycles":
        out = sorted((n for n in nodes if n in reach(needs, n)), key=key)
    else:
        left = {n: len(needs[n]) for n in nodes}; out = []
        ready = [n for n in nodes if left[n] == 0]
        while ready:
            ready.sort(key=key); n = ready.pop(0); out.append(n)
            for d in needed[n]:
                left[d] -= 1
                if left[d] == 0: ready.append(d)
        if len(out) < len(nodes):
            sys.stderr.write("error: cycle\n"); return 3
    sys.stdout.write("".join(x + "\n" for x in out))
    return 2 if bad else 0

if __name__ == "__main__": sys.exit(main(sys.argv[1:]))
