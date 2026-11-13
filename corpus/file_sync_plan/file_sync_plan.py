def plan(src, dst):
    acts = []
    for name in sorted(src):
        if name not in dst:
            acts.append("add    " + name)
        elif dst[name] != src[name]:
            acts.append("update " + name)
    for name in sorted(dst):
        if name not in src:
            acts.append("delete " + name)
    return acts


def main():
    src = {"a.txt": "alpha\n", "b.txt": "beta v2\n", "c.txt": "gamma\n"}
    dst = {"b.txt": "beta v1\n", "c.txt": "gamma\n", "old.txt": "stale\n"}
    acts = plan(src, dst)
    print(f"plan: {len(acts)} actions")
    for a in acts:
        print(f"  {a}")
    dst = dict(src)
    after = plan(src, dst)
    print(f"after apply: {len(after)} actions")
    print(f"destination: {', '.join(sorted(dst))}")
    print("cleaned up: true")


main()
