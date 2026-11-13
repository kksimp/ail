DR = [0, 1, 1, 1, 0, -1, -1, -1]
DC = [1, 1, 0, -1, -1, -1, 0, 1]
NAMES = ["E", "SE", "S", "SW", "W", "NW", "N", "NE"]


def fits(cells, n, word, r, c, d, exact):
    last_r = r + DR[d] * (len(word) - 1)
    last_c = c + DC[d] * (len(word) - 1)
    if last_r < 0 or last_r >= n or last_c < 0 or last_c >= n:
        return False
    for k in range(len(word)):
        have = cells[(r + DR[d] * k) * n + c + DC[d] * k]
        if have != word[k] and (exact or have != ""):
            return False
    return True


def place(cells, n, text):
    for r in range(n):
        for c in range(n):
            for d in range(8):
                if fits(cells, n, text, r, c, d, False):
                    for k in range(len(text)):
                        cells[(r + DR[d] * k) * n + c + DC[d] * k] = text[k]
                    return True
    return False


def fill_rest(cells, seed):
    letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    state = seed
    for i in range(len(cells)):
        state = (state * 1103515245 + 12345) % 2147483648
        if cells[i] == "":
            cells[i] = letters[(state // 65536) % 26]


def solve(cells, n, text):
    for r in range(n):
        for c in range(n):
            for d in range(8):
                if fits(cells, n, text, r, c, d, True):
                    return (r, c, d)
    return None


def main():
    n = 10
    words = ["VOLTARE", "STRING", "TRAIT", "ENUM", "CHECK", "GRID", "LIST", "ALE"]
    cells = [""] * (n * n)
    placed = 0
    for w in words:
        if place(cells, n, w):
            placed += 1
    fill_rest(cells, 2024)
    print(f"placed {placed} of {len(words)} words")
    for r in range(n):
        print(" ".join(cells[r * n + c] for c in range(n)))
    print("solutions:")
    for w in words:
        hit = solve(cells, n, w)
        if hit is None:
            print(f"  {w:<8} not found")
            continue
        print(f"  {w:<8} row {hit[0] + 1} col {hit[1] + 1} {NAMES[hit[2]]}")


main()
