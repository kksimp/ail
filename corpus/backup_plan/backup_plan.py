import hashlib, os, shutil, tempfile


def snapshot(d):
    snap = {}
    for name in sorted(os.listdir(d)):
        with open(os.path.join(d, name), newline="") as f:
            snap[name] = hashlib.sha256(f.read().encode()).hexdigest()[:8]
    return snap


def make_plan(before, after):
    steps = []
    for name in sorted(after):
        if name not in before:
            steps.append(f"add {name}")
        elif before[name] != after[name]:
            steps.append(f"update {name}")
    for name in sorted(before):
        if name not in after:
            steps.append(f"delete {name}")
    return steps


def apply_plan(steps, src, dest):
    for step in steps:
        verb, name = step.split(" ")
        target = os.path.join(dest, name)
        if verb == "delete":
            os.remove(target)
        else:
            shutil.copyfile(os.path.join(src, name), target)


def write(d, name, text):
    with open(os.path.join(d, name), "w", newline="") as f:
        f.write(text)


def main():
    root = tempfile.mkdtemp(prefix="corpus_backup_plan")
    src = os.path.join(root, "src")
    dest = os.path.join(root, "dest")
    os.mkdir(src)
    os.mkdir(dest)
    for n, t in zip(["notes.txt", "todo.txt", "old.log", "data.csv"], ["meeting at 10\n", "buy milk\n", "boot ok\n", "a,b\n1,2\n"]):
        write(src, n, t)
    for run_no in (1, 2):
        print(f"== backup {run_no} ==")
        now = snapshot(src)
        before = snapshot(dest)
        plan = make_plan(before, now)
        for step in plan:
            print(f"  {step}")
        same = [n for n in now if before.get(n) == now[n]]
        print(f"  {len(plan)} actions, {len(same)} unchanged")
        apply_plan(plan, src, dest)
        after = snapshot(dest)
        print(f"  verified: {str(after == now).lower()}")
        if run_no == 1:
            write(src, "todo.txt", "buy milk and eggs\n")
            os.remove(os.path.join(src, "old.log"))
            write(src, "new.md", "# new\n")
    shutil.rmtree(root)


main()
