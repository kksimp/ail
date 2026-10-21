import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
DATA = ROOT / "tools/tokbench/corpus/agent-r5/data"
AIL = (shutil.which("ail") or str(ROOT / "ail"))


def ail(*args, stdin=None):
    return subprocess.run([AIL, *args], capture_output=True, text=True, input=stdin)


for file in [DATA / "pkg/mathx.ail", DATA / "ok.ail"]:
    r = ail("api", str(file))
    doc = json.loads(r.stdout)
    for item in doc.get("items") or []:
        if (item.get("kind") or "") == "fn":
            documented = "true" if len(item.get("doc") or "") > 0 else "false"
            print(f"{file.name} {item.get('name') or ''} doc={documented} {item.get('signature') or ''}")
