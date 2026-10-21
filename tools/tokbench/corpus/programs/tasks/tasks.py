import json
import subprocess
from pathlib import Path

MAX_HOURS = 3.0
DATA = Path(__file__).resolve().parents[5] / "examples/data"


class TaskError(Exception):
    def __init__(self, id, field):
        self.id = id
        self.field = field


def load_task(j):
    for field in ("id", "title", "status"):
        if not isinstance(j.get(field), str):
            raise TaskError(j.get("id", "?") if field != "id" else "?", field)
    return {**j, "hours": float(j.get("hours", 0))}


def with_status(j, status):
    return {k: status if k == "status" else v for k, v in j.items()}


def main():
    doc = json.loads((DATA / "tasks.json").read_text())
    updated = []
    ran = failed = 0
    for item in doc.get("tasks", []):
        try:
            task = load_task(item)
        except TaskError as err:
            print(f"skip {err.id}: missing {err.field}")
            updated.append(item)
            continue
        if task["status"] != "todo" or task["hours"] > MAX_HOURS:
            print(f"keep {task['id']}: {task['title']} ({task['status']}, {task['hours']}h)")
            updated.append(item)
            continue
        r = subprocess.run(["echo", "running", task["title"]], capture_output=True, text=True)
        ran += 1
        failed += r.returncode != 0
        print(f"{task['id']}: {r.stdout.strip()}")
        updated.append(with_status(item, "done" if r.returncode == 0 else "failed"))
    out = json.dumps({"tasks": updated}, separators=(",", ":"), ensure_ascii=False)
    path = DATA / "tasks.out.json"
    path.write_text(out)
    back = json.loads(path.read_text())
    path.unlink()
    print(f"ran {ran}, failed {failed}, wrote {len(back['tasks'])} tasks ({len(out)} chars)")


def test_loads_a_task():
    t = load_task({"id": "a", "title": "t", "status": "todo", "hours": 2.0})
    assert t["title"] == "t" and t["hours"] == 2.0
    try:
        load_task({"id": "a", "status": "todo"})
    except TaskError as err:
        assert err.field == "title"
    else:
        raise AssertionError("expected a TaskError")


def test_sets_status():
    assert json.dumps(with_status({"status": "todo", "n": 1.0}, "done"), separators=(",", ":")) == '{"status":"done","n":1.0}'


if __name__ == "__main__":
    main()
