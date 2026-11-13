import os, shutil, tempfile


def parse_todos(text):
    out = []
    for line in text.splitlines():
        cells = line.split("|")
        if len(cells) != 3:
            continue
        if not cells[0].lstrip("-").isdigit():
            continue
        out.append((int(cells[0]), cells[1] == "1", cells[2]))
    return out


def render_todos(todos):
    return "\n".join(f"{i}|{'1' if d else '0'}|{t}" for i, d, t in todos)


def complete(todos, id):
    return [(i, d or i == id, t) for i, d, t in todos]


def drop(todos, id):
    return [x for x in todos if x[0] != id]


def load(file):
    if not os.path.exists(file):
        return []
    with open(file, newline="") as f:
        return parse_todos(f.read())


def run(file, line):
    todos = load(file)
    parts = line.split(" ", 1)
    verb = parts[0]
    arg = parts[1] if len(parts) > 1 else ""
    ids = [t[0] for t in todos]
    if verb == "add":
        nxt = (max(ids) if ids else 0) + 1
        todos.append((nxt, False, arg))
        print(f"added {nxt}: {arg}")
    elif verb == "done" or verb == "rm":
        id = int(arg) if arg.lstrip("-").isdigit() and arg != "" else -1
        if id not in ids:
            print(f"no such todo: {arg}")
            return
        if verb == "done":
            todos = complete(todos, id)
            print(f"completed {id}")
        else:
            todos = drop(todos, id)
            print(f"removed {id}")
    elif verb == "list":
        if not todos:
            print("nothing to do")
        for i, d, t in todos:
            mark = "x" if d else " "
            print(f"[{mark}] {i} {t}")
        open_ = [t for t in todos if not t[1]]
        print(f"{len(open_)} open, {len(todos) - len(open_)} done")
    else:
        print(f"unknown command: {verb}")
    with open(file, "w", newline="") as f:
        f.write(render_todos(todos))


def main():
    d = tempfile.mkdtemp(prefix="corpus_todo_cli")
    file = os.path.join(d, "todos.txt")
    commands = [
        "list",
        "add buy milk",
        "add write the report",
        "add call mom",
        "list",
        "done 2",
        "done 9",
        "rm 1",
        "add water plants",
        "frobnicate now",
        "list",
    ]
    for c in commands:
        print(f"> {c}")
        run(file, c)
    with open(file, newline="") as f:
        print(f"file holds {len(f.read().splitlines())} lines")
    shutil.rmtree(d)


main()
