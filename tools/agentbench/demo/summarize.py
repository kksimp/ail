"""Summarize an agent transcript (claude -p --output-format stream-json) as a short markdown report:
the commands it ran, how many turns and tokens it used, and its last words. Usage: summarize.py TRANSCRIPT"""
import json, sys

cmds, last, res = [], "", {}
for line in open(sys.argv[1]):
    try:
        e = json.loads(line)
    except ValueError:
        continue
    if e.get("type") == "assistant":
        for c in e.get("message", {}).get("content", []):
            if c.get("type") == "tool_use":
                i = c.get("input", {})
                what = i.get("command") or i.get("file_path") or i.get("pattern") or ""
                cmds.append((c.get("name", "?"), " ".join(str(what).split())[:140]))
            elif c.get("type") == "text":
                last = c.get("text", "").strip()
    elif e.get("type") == "result":
        res = e
u = res.get("usage", {})
print("## Agent run")
print(f"- turns {res.get('num_turns', '?')}, tool calls {len(cmds)}, output tokens {u.get('output_tokens', '?')}, "
      f"input {u.get('input_tokens', '?')} + cache read {u.get('cache_read_input_tokens', '?')}, "
      f"cost ${res.get('total_cost_usd', 0):.2f}, {res.get('duration_ms', 0) // 1000} s")
print("- commands:")
for n, w in cmds:
    print(f"  - {n}: {w}")
print(f"- last words: {last[:400]}")
