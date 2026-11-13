def plan(names, prefix, ext):
    picked = [n for n in sorted(names) if n.endswith("." + ext)]
    return [(src, f"{prefix}_{i + 1:03}.{ext}") for i, src in enumerate(picked)]


def conflicts(moves, names):
    return [m[1] + ".tmp" for m in moves if m[0] != m[1] and (m[1] + ".tmp") in names]


def run_batch(files, prefix):
    names = list(files)
    moves = plan(names, prefix, "jpg")
    bad = conflicts(moves, names)
    print(f"dry run, {len(moves)} files:")
    for src, dst in moves:
        mark = "same" if src == dst else "move"
        print(f"  {mark} {src:<14}-> {dst}")
    if bad:
        print(f"conflict with stale files: {', '.join(bad)}; nothing changed")
        return
    changed = 0
    staged = {}
    for src, dst in moves:
        if src != dst:
            staged[dst] = files.pop(src)
    for dst, content in staged.items():
        files[dst] = content
        changed += 1
    print(f"renamed {changed} files")


def main():
    files = {}
    for n in ["beach.jpg", "arch.jpg", "notes.txt", "dune.jpg", "cafe.jpg", "map.png"]:
        files[n] = n + "\n"
    print("first run:")
    run_batch(files, "trip")
    print(f"now: {' '.join(sorted(files))}")
    print("second run:")
    run_batch(files, "trip")
    files["snap.jpg"] = "snap\n"
    files["trip_001.jpg.tmp"] = "stale\n"
    print("third run with a stale temp file:")
    run_batch(files, "trip")
    del files["trip_001.jpg.tmp"]
    print("fourth run after removing it:")
    run_batch(files, "trip")
    print(f"final: {' '.join(sorted(files))}")
    print(f"trip_001 holds: {files['trip_001.jpg'].strip()}")
    print("cleaned up: true")


main()
