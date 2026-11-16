#!/usr/bin/env python3
"""The in-workspace visible-case runner behind ./test (ADR 0916). prep() copies it into the workspace's .bench/, so the
workspace names no harness path and the hidden cases are not one directory away (E14 review). Usage: visible.py DIR"""
import json, os, subprocess, sys, time

def run_case(exe, case, timeout=10):
    t0 = time.time()
    try:
        p = subprocess.run([exe] + case["args"], input=case["stdin"].encode(), capture_output=True, timeout=timeout)
        err_ok = "stderr" not in case or p.stderr.decode("utf-8", "replace") == case["stderr"]  # tasks with "check_stderr" also fix stderr
        return {"name": case["name"], "ok": p.stdout.decode("utf-8", "replace") == case["stdout"] and p.returncode == case["code"] and err_ok,
                "code": p.returncode, "stdout": p.stdout.decode("utf-8", "replace"), "err": p.stderr.decode("utf-8", "replace")[-400:], "s": round(time.time() - t0, 3)}
    except subprocess.TimeoutExpired:
        return {"name": case["name"], "ok": False, "code": None, "stdout": "", "err": "timeout", "s": timeout}

def main(d):
    with open(os.path.join(d, ".bench", "visible.jsonl")) as f: cases = [json.loads(l) for l in f if l.strip()]
    exe = os.path.join(d, open(os.path.join(d, ".bench", "exe")).read()); bad = 0
    for c in cases:
        r = run_case(exe, c)
        if not r["ok"]:
            bad += 1; print(f"FAIL {c['name']}: args={c['args']} want code {c['code']} got {r['code']}\n  want stdout {c['stdout']!r}\n  got  stdout {r['stdout']!r}\n"
                            + (f"  want stderr {c['stderr']!r}\n" if "stderr" in c else "") + f"  got  stderr {r['err']!r}")
    print(f"{len(cases) - bad}/{len(cases)} visible cases pass"); return 1 if bad else 0

if __name__ == "__main__": sys.exit(main(sys.argv[1] if len(sys.argv) > 1 else "."))
