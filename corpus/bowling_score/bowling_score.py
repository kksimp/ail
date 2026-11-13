def parse_rolls(game):
    rolls = []
    for ch in game.replace(" ", ""):
        if ch == "X":
            rolls.append(10)
        elif ch == "/":
            prev = rolls[-1]
            rolls.append(10 - prev)
        elif ch == "-":
            rolls.append(0)
        else:
            rolls.append(int(ch))
    return rolls


def running(rolls):
    totals = []
    total = 0
    at = 0
    for _ in range(10):
        if rolls[at] == 10:
            total += 10 + rolls[at + 1] + rolls[at + 2]
            at += 1
        elif rolls[at] + rolls[at + 1] == 10:
            total += 10 + rolls[at + 2]
            at += 2
        else:
            total += rolls[at] + rolls[at + 1]
            at += 2
        totals.append(total)
    return totals


def describe(label, game):
    totals = running(parse_rolls(game))
    print(f"{label}: {game}")
    print("  frames: " + " ".join(str(t) for t in totals))
    print(f"  strikes {game.count('X')}, spares {game.count('/')}, score {totals[9]}")


def main():
    describe("perfect game", "X X X X X X X X X X X X")
    describe("gutter game", "-- -- -- -- -- -- -- -- -- --")
    describe("nines", "9- 9- 9- 9- 9- 9- 9- 9- 9- 9-")
    describe("all spares", "5/ 5/ 5/ 5/ 5/ 5/ 5/ 5/ 5/ 5/5")
    describe("mixed game", "X 7/ 9- X -8 8/ -6 X X X81")
    describe("open frames", "72 35 8- 44 61 17 53 90 26 81")


main()
