def roster():
    names = ["ana", "ben", "cy", "dee", "eli", "fay", "gus", "hal", "ivy", "jon"]
    raws = [48, 62, 55, 71, 39, 66, 58, 80, 52, 60]
    return list(zip(names, raws))


def letter(score):
    if score >= 90:
        return "A"
    if score >= 80:
        return "B"
    if score >= 70:
        return "C"
    if score >= 60:
        return "D"
    return "F"


def curve_shift(scores, target):
    return target - sum(scores) // len(scores)


def curved(score, shift):
    return max(min(score + shift, 100), 0)


def percentile(sorted_xs, p):
    n = len(sorted_xs)
    idx = (p * n + 99) // 100 - 1
    return sorted_xs[max(idx, 0)]


def tally(scores):
    out = {}
    for s in scores:
        out[letter(s)] = out.get(letter(s), 0) + 1
    return out


def main():
    pupils = roster()
    raws = [p[1] for p in pupils]
    shift = curve_shift(raws, 75)
    print(f"mean {sum(raws) // len(raws)}, shift {shift}")
    ordered = sorted(raws)
    print(f"quartiles {percentile(ordered, 25)} {percentile(ordered, 50)} {percentile(ordered, 75)}")
    print("name  raw  new  grade")
    bumped = []
    for p in sorted(pupils, key=lambda x: -x[1]):
        c = curved(p[1], shift)
        bumped.append(c)
        print(f"{p[0]:<5} {p[1]:>3} {c:>4}  {letter(p[1])}->{letter(c)}")
    before_counts = tally(raws)
    after_counts = tally(bumped)
    for g in ["A", "B", "C", "D", "F"]:
        b = before_counts.get(g, 0)
        a = after_counts.get(g, 0)
        print(f"{g}: {b} -> {a}")


main()
