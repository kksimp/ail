CANDS = ["Ada", "Ben", "Cyd", "Dee"]


class VoteError(Exception):
    pass


def parse_ballot(text):
    names = text.split(">")
    seen = []
    for n in names:
        if n not in CANDS or n in seen:
            raise VoteError(f"bad ballot '{text}'")
        seen.append(n)
    return seen


def leader(score):
    best = 0
    for s in score:
        best = max(best, s)
    names = [CANDS[i] for i in range(len(score)) if score[i] == best]
    if len(names) > 1:
        raise VoteError("tie between " + "/".join(names))
    return names[0]


def plurality(ballots):
    if not ballots:
        raise VoteError("no ballots")
    score = [0] * len(CANDS)
    for b in ballots:
        score[CANDS.index(b[0])] += 1
    return leader(score)


def borda(ballots):
    if not ballots:
        raise VoteError("no ballots")
    score = [0] * len(CANDS)
    for b in ballots:
        for r in range(len(b)):
            score[CANDS.index(b[r])] += len(CANDS) - 1 - r
    return leader(score)


def runoff(ballots):
    if not ballots:
        raise VoteError("no ballots")
    alive = [True] * 4
    left = len(CANDS)
    while left > 1:
        counts = [0] * len(CANDS)
        active = 0
        for b in ballots:
            for name in b:
                i = CANDS.index(name)
                if alive[i]:
                    counts[i] += 1
                    active += 1
                    break
        low = -1
        for k in range(len(CANDS)):
            if alive[k] and counts[k] * 2 > active:
                return CANDS[k]
            if alive[k] and (low < 0 or counts[k] < counts[low]):
                low = k
        alive[low] = False
        left -= 1
    return CANDS[alive.index(True)]


def report(title, texts):
    print(title)
    try:
        ballots = [parse_ballot(t) for t in texts]
    except VoteError as e:
        print(f"  {e}")
        return
    for label, m in (("plurality", plurality), ("borda", borda), ("runoff", runoff)):
        try:
            w = m(ballots)
        except VoteError as e:
            print(f"  {label}: {e}")
            continue
        print(f"  {label}: {w}")


report("box one", ["Ada>Ben>Cyd", "Ada>Cyd", "Ben>Cyd>Ada", "Ben>Cyd>Ada", "Cyd>Ben>Ada",
                   "Cyd>Ben", "Dee>Cyd>Ben", "Cyd>Dee>Ben>Ada", "Ada>Dee", "Ben>Ada"])
report("box two", ["Ada>Ben", "Ben>Ada", "Cyd>Dee", "Dee>Cyd"])
report("box three", ["Ada>Ben", "Ada>Zed"])
report("box four", [])
