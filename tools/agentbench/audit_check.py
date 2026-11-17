"""Unit check for the audit's void rule (ADR 0916, amended 2026-11-17): python3 -I tools/agentbench/audit_check.py"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import agentbench as a

ws = "/private/tmp/trial-a/ws"; homes = {ws, "/tmp/trial-a/ws"}
own = lambda q: any(q == h or q.startswith(h.rstrip("/") + "/") for h in homes)
CASES = [("cd /private/tmp/trial-a/ws/src; cat x; ls ../.bench", False),
         ("cd /private/tmp/trial-a/ws/src; head -c 600 ../.bench/visible.jsonl", False),
         ("cd /private/tmp/trial-a/ws; head ../../trial-b/ws/src/main.ail", True),
         ("cat ../../ail-heldout/calc/cases/hidden.jsonl", True),
         ("cat .bench/visible.jsonl", False)]
bad = [t for t, want in CASES if a.void_text(t, homes, own) != want]
print("audit_check: " + ("ok" if not bad else "FAILED: " + "; ".join(bad))); sys.exit(1 if bad else 0)
