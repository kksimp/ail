LINES = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]]


def winner(board):
    for line in LINES:
        a = board[line[0]]
        if a != 0 and a == board[line[1]] and a == board[line[2]]:
            return a
    return 0


def is_full(board):
    for cell in board:
        if cell == 0:
            return False
    return True


def other(me):
    return 3 - me


def negamax(board, me):
    w = winner(board)
    if w != 0:
        return 1 if w == me else -1
    if is_full(board):
        return 0
    best = -2
    for i in range(9):
        if board[i] == 0:
            after = list(board)
            after[i] = me
            v = 0 - negamax(after, other(me))
            if v > best:
                best = v
    return best


def best_move(board, me):
    best = -2
    pick = -1
    for i in range(9):
        if board[i] == 0:
            after = list(board)
            after[i] = me
            v = 0 - negamax(after, other(me))
            if v > best:
                best = v
                pick = i
    return pick


def tree_size(board, me):
    if winner(board) != 0 or is_full(board):
        return 1
    total = 1
    for i in range(9):
        if board[i] == 0:
            after = list(board)
            after[i] = me
            total += tree_size(after, other(me))
    return total


def render(board):
    marks = [".", "X", "O"]
    rows = []
    for r in range(3):
        c0 = marks[board[r * 3]]
        c1 = marks[board[r * 3 + 1]]
        c2 = marks[board[r * 3 + 2]]
        rows.append(f"{c0}{c1}{c2}")
    return rows


def show(title, board):
    print(title)
    for row in render(board):
        print(f"  {row}")


def main():
    empty = [0, 0, 0, 0, 0, 0, 0, 0, 0]
    print(f"game tree nodes from the empty board: {tree_size(empty, 1)}")
    board = list(empty)
    me = 1
    moves = []
    while winner(board) == 0 and not is_full(board):
        spot = best_move(board, me)
        board[spot] = me
        moves.append(spot)
        me = other(me)
    print("self-play moves: " + " ".join(str(x) for x in moves))
    show(f"final board, winner {winner(board)}:", board)
    take_win = [1, 1, 0, 2, 2, 0, 0, 0, 0]
    print(f"X to move with a win available: plays {best_move(take_win, 1)}")
    must_block = [1, 1, 0, 0, 2, 0, 0, 0, 0]
    print(f"O to move, X threatens 2: plays {best_move(must_block, 2)}")
    corner_trap = [1, 0, 0, 0, 2, 0, 0, 0, 1]
    print(f"O to move after X takes opposite corners: plays {best_move(corner_trap, 2)}")


main()
