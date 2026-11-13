def lines():
    return [0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 3, 6, 1, 4, 7, 2, 5, 8, 0, 4, 8, 2, 4, 6]


def winner(board):
    ls = lines()
    for k in range(8):
        total = board[ls[3 * k]] + board[ls[3 * k + 1]] + board[ls[3 * k + 2]]
        if total == 3:
            return 1
        if total == -3:
            return -1
    return 0


def read_board(text):
    out = []
    for ch in text:
        if ch == "X":
            out.append(1)
        elif ch == "O":
            out.append(-1)
        else:
            out.append(0)
    return out


def plain(board, player):
    w = winner(board)
    if w != 0:
        return (w, 1)
    nodes = 1
    moved = False
    best = -2 if player == 1 else 2
    for i in range(9):
        if board[i] != 0:
            continue
        moved = True
        nxt = list(board)
        nxt[i] = player
        score, n = plain(nxt, -player)
        nodes += n
        if player == 1 and score > best:
            best = score
        if player == -1 and score < best:
            best = score
    if not moved:
        best = 0
    return (best, nodes)


def pruned(board, player, alpha, beta):
    w = winner(board)
    if w != 0:
        return (w, 1)
    nodes = 1
    moved = False
    lo = alpha
    hi = beta
    best = -2 if player == 1 else 2
    for i in range(9):
        if board[i] != 0:
            continue
        moved = True
        nxt = list(board)
        nxt[i] = player
        score, n = pruned(nxt, -player, lo, hi)
        nodes += n
        if player == 1:
            if score > best:
                best = score
            if best > lo:
                lo = best
        else:
            if score < best:
                best = score
            if best < hi:
                hi = best
        if lo >= hi:
            break
    if not moved:
        best = 0
    return (best, nodes)


def to_move(board):
    return 1 if sum(board) == 0 else -1


def best_move(board, player):
    pick = -1
    pick_score = 0
    for i in range(9):
        if board[i] == 0:
            nxt = list(board)
            nxt[i] = player
            score, _ = pruned(nxt, -player, -2, 2)
            if pick < 0 or score * player > pick_score * player:
                pick = i
                pick_score = score
    return pick


def verdict(score):
    if score == 1:
        return "X wins"
    if score == -1:
        return "O wins"
    return "draw"


for pos in [".........", "XX.OO....", "X...O....", "XO.XO....", "XOXOXO..."]:
    board = read_board(pos)
    side = to_move(board)
    slow = plain(board, side)
    fast = pruned(board, side, -2, 2)
    print(f"{pos}: {verdict(fast[0])}, move {best_move(board, side)}, nodes {slow[1]} vs {fast[1]}")
game = read_board(".........")
side_now = 1
moves = []
while winner(game) == 0 and 0 in game:
    m = best_move(game, side_now)
    game[m] = side_now
    moves.append(m)
    side_now = -side_now
print(f"self-play moves: {moves}")
print(f"result: {verdict(winner(game))}")
