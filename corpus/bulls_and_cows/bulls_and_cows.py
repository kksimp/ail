def all_codes():
    out = []
    for a in range(10):
        for b in range(10):
            for c in range(10):
                for d in range(10):
                    if a != b and a != c and a != d and b != c and b != d and c != d:
                        out.append(f"{a}{b}{c}{d}")
    return out


def score(secret, guess):
    bulls = 0
    common = 0
    for i in range(4):
        if secret[i] == guess[i]:
            bulls += 1
        if guess[i] in secret:
            common += 1
    return (bulls, common - bulls)


def show(fb):
    return f"{fb[0]}B {fb[1]}C"


def solve(secret):
    cands = all_codes()
    rounds = 0
    print(f"secret {secret}:")
    while rounds < 12:
        guess = cands[0]
        fb = score(secret, guess)
        rounds += 1
        cands = [c for c in cands if score(c, guess) == fb]
        print(f"  {rounds}. guess {guess} -> {show(fb)}, {len(cands)} left")
        if fb[0] == 4:
            break
    return rounds


def main():
    print(f"possible codes: {len(all_codes())}")
    total = 0
    for secret in ["4271", "9035", "0123", "9876"]:
        total += solve(secret)
    print(f"total guesses: {total}")


main()
