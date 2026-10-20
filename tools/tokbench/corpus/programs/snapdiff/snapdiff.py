from pathlib import Path

DATA = Path(__file__).resolve().parents[5] / "examples/data"


def snapshot(dir):
    return {p.name: p.read_text().splitlines() for p in dir.iterdir()}


def diff(before, after):
    out = [f"- {line}" for line in before if line not in after]
    out += [f"+ {line}" for line in after if line not in before]
    return out


def main():
    old = snapshot(DATA / "snap_old")
    new = snapshot(DATA / "snap_new")
    changed = 0
    for name in sorted(old.keys() | new.keys()):
        if name not in old:
            print(f"added {name} ({len(new[name])} lines)")
            continue
        if name not in new:
            print(f"removed {name}")
            continue
        lines = diff(old[name], new[name])
        if lines:
            changed += 1
            print(f"changed {name}")
            for line in lines:
                print(f"  {line}")
    print(f"{changed} changed, {len(new)} files now")


def test_diffs_lines():
    assert diff(["a", "b"], ["b", "c"]) == ["- a", "+ c"]
    assert diff(["a"], ["a"]) == []


if __name__ == "__main__":
    main()
