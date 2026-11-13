def key_of(line, fold):
    t = line.strip()
    return t.lower() if fold else t


def dedupe(lines, fold):
    seen = set()
    kept = []
    for line in lines:
        k = key_of(line, fold)
        if line.strip() == "" or k not in seen:
            seen.add(k)
            kept.append(line)
    return kept


def repeats(lines, fold):
    best = 0
    for a in lines:
        if a.strip() == "":
            continue
        n = sum(1 for b in lines if key_of(a, fold) == key_of(b, fold))
        best = max(best, n)
    return best


def main():
    text = "apple\nBanana\napple\n\ncherry\n  banana \n\nApple\ndate\ncherry\n"
    lines = text.split("\n")[:-1]
    for label, fold in [("exact", False), ("folded", True)]:
        kept = dedupe(lines, fold)
        print(f"{label}: read {len(lines)}, kept {len(kept)}, dropped {len(lines) - len(kept)}")
        print("  " + "|".join(kept))
        print(f"  most repeated line appears {repeats(lines, fold)} times")
    print("cleaned up: true")


main()
