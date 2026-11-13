def parse_commit(line):
    idx = line.find(": ")
    if idx < 0:
        return None
    head = line[:idx]
    desc = line[idx + 2:].strip()
    breaking = head.endswith("!")
    core = head[:-1] if breaking else head
    kind = core
    scope = ""
    lp = core.find("(")
    if lp >= 0:
        if not core.endswith(")"):
            return None
        kind = core[:lp]
        scope = core[lp + 1:len(core) - 1]
    if kind == "" or desc == "":
        return None
    return (kind, scope, desc, breaking)


def section_title(kind):
    return {"feat": "Features", "fix": "Bug Fixes", "perf": "Performance", "docs": "Documentation"}.get(kind, "")


def bump(version, commits):
    parts = version.split(".")
    major, minor, patch = int(parts[0]), int(parts[1]), int(parts[2])
    any_break = any(c[3] for c in commits)
    any_feat = any(c[0] == "feat" for c in commits)
    if any_break:
        major += 1
        minor = 0
        patch = 0
    elif any_feat:
        minor += 1
        patch = 0
    else:
        patch += 1
    return f"{major}.{minor}.{patch}"


def build(version, subjects):
    commits = []
    for s in subjects:
        c = parse_commit(s)
        if c is None:
            continue
        commits.append(c)
    skipped = len(subjects) - len(commits)
    nxt = bump(version, commits)
    out = [f"## {nxt}"]
    breaking = [c[2] for c in commits if c[3]]
    if breaking:
        out.append("")
        out.append("### BREAKING CHANGES")
        for b in breaking:
            out.append(f"- {b}")
    for kind in ["feat", "fix", "perf", "docs"]:
        rows = []
        for c in commits:
            if c[0] == kind:
                tag = "" if c[1] == "" else f"**{c[1]}:** "
                rows.append(f"- {tag}{c[2]}")
        if rows:
            out.append("")
            out.append(f"### {section_title(kind)}")
            out.extend(rows)
    out.append("")
    out.append(f"({len(commits)} commits, {skipped} skipped)")
    return out


def main():
    subjects = [
        "feat(parser): accept trailing commas",
        "fix(fmt): keep blank line after imports",
        "chore: bump seed",
        "feat!: drop the old module syntax",
        "docs(spec): clarify shadowing",
        "fix: crash on empty file",
        "perf(check): cache lookups",
        "update stuff",
        "fix(: broken scope",
    ]
    for line in build("1.4.2", subjects):
        print(line)
    print("---")
    for line in build("2.0.9", ["fix: one", "fix(io): two"]):
        print(line)


main()
