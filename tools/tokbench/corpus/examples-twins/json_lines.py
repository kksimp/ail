# Python twin of examples/json_lines.ail: JSON lines, bad lines skipped, defaults.
import json

text = "\n".join([
    '{"level": "warn", "msg": "disk 91%", "retry": true}',
    "not json at all",
    '{"level": "error", "msg": "write failed"}',
    '{"level": "info", "msg": "done", "retry": false}',
])

events = []
for line in text.split("\n"):
    try:
        events.append(json.loads(line))
    except ValueError:
        continue

retries = 0
for e in events:
    level = e.get("level", "?")
    msg = e.get("msg", "")
    if e.get("retry", False) is True:
        retries += 1
    print(f"{level}: {msg}")
print(f"{len(events)} events, {retries} retried")
