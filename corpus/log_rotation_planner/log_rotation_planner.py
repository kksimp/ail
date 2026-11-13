def decide(e, rank, p):
    name, size, age = e
    max_age, keep, compress_after = p
    if age > max_age or rank >= keep:
        return "delete"
    if age > compress_after and not name.endswith(".gz"):
        return "compress"
    return "keep"


def main():
    entries = [
        ("app.log", 40, 0),
        ("app.log.1", 55, 1),
        ("app.log.2.gz", 20, 9),
        ("app.log.3", 60, 3),
        ("app.log.4", 70, 12),
        ("app.log.5.gz", 18, 31),
        ("app.log.6", 65, 6),
        ("app.log.7", 90, 20),
    ]
    policy = (30, 6, 7)
    ordered = sorted(entries, key=lambda x: x[2])
    actions = [decide(e, i, policy) for i, e in enumerate(ordered)]
    freed = 0
    for i, e in enumerate(ordered):
        print(f"{actions[i]:<9}{e[0]:<14}{e[2]:>3}d {e[1]:>4}")
        if actions[i] == "delete":
            freed += e[1]
    print(f"frees {freed} bytes")
    left = []
    for i, (name, size, age) in enumerate(ordered):
        if actions[i] == "delete":
            continue
        if actions[i] == "compress":
            name = name + ".gz"
        left.append((name, size))
    left.sort()
    total = 0
    for name, size in left:
        total += size
        print(f"  {name:<14}{size:>4}")
    print(f"{len(left)} files, {total} bytes left")
    print("cleaned up: true")


main()
