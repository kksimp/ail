#!/usr/bin/env python3
"""What the C arm of a task is handed, as one hash per task, for the rerun's reuse of saved C trials (plans/rerun.txt, `report --reuse-c`).

usage: c_fingerprint.py TOOLS_AGENTBENCH_DIR TASK...     (no model, no spend, no AIL build)
Imports agentbench from the given tools/agentbench folder (an old tree's or the current one's), prepares the C workspace of each task in a
temp folder with that module's own `prep`, and prints `TASK sha256 docs_bytes`: the hash covers every file of the workspace (SPEC.md, DOCS/,
seed, shims, visible cases, build and test scripts; not .git or the machine paths of .bench/bin), the whole task folder (hidden cases, task.json,
oracle) and the read-policy prompt. docs_bytes is the size of DOCS/ as the agent receives it, to compare with `docs_bytes` in the old rows.
The two trees give the same hash exactly when nothing the C agent sees or is scored by has changed."""
import hashlib, os, sys, tempfile

def tree_hash(root, skip=()):
    h = hashlib.sha256()
    for dp, dn, fs in os.walk(root):
        dn[:] = sorted(d for d in dn if d not in (".git",) and os.path.relpath(os.path.join(dp, d), root) not in skip)
        for f in sorted(fs):
            p = os.path.join(dp, f)
            if os.path.relpath(p, root) in skip: continue
            h.update(os.path.relpath(p, root).encode() + b"\0")
            h.update(open(p, "rb").read() if not os.path.islink(p) else os.readlink(p).encode()); h.update(b"\0")
    return h.hexdigest()

def main(argv):
    if len(argv) < 2: sys.exit(__doc__)
    tools = os.path.abspath(argv[0]); os.environ.pop("AGENTBENCH_HELDOUT", None)   # practised tasks live in the tree, never in the held-out folder
    sys.path.insert(0, tools); import agentbench as a
    for t in argv[1:]:
        task = a.load_task(t); ws = os.path.join(tempfile.mkdtemp(prefix="cfp-"), "ws")
        a.prep(t, task, "c", ws)
        docs = sum(os.path.getsize(os.path.join(ws, "DOCS", f)) for f in os.listdir(os.path.join(ws, "DOCS")))
        h = hashlib.sha256()
        h.update(tree_hash(ws, skip=(os.path.join(".bench", "bin"),)).encode()); h.update(tree_hash(a.task_dir(t)).encode())
        h.update(a.PROMPT.encode()); h.update(a.SHIM_BUILD.encode()); h.update(a.SHIM_TEST.encode())
        print(t, h.hexdigest(), docs)

if __name__ == "__main__": main(sys.argv[1:])
