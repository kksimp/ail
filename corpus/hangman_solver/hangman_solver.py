ALPHABET = "abcdefghijklmnopqrstuvwxyz"


def fits(word, pattern, guessed):
    if len(word) != len(pattern):
        return False
    for i in range(len(word)):
        if pattern[i] != '_':
            if word[i] != pattern[i]:
                return False
        elif word[i] in guessed:
            return False
    return True


def best_letter(words, guessed):
    best = '?'
    best_count = 0
    for letter in ALPHABET:
        if letter in guessed:
            continue
        n = 0
        for word in words:
            if letter in word:
                n += 1
        if n > best_count:
            best_count = n
            best = letter
    return best


def play(secret, dictionary):
    pattern = ['_'] * len(secret)
    guessed = []
    words = [w for w in dictionary if len(w) == len(secret)]
    wrong = 0
    print(f"secret has {len(secret)} letters, {len(words)} candidates")
    while '_' in pattern and wrong < 6:
        g = best_letter(words, guessed)
        guessed.append(g)
        if g in secret:
            for i in range(len(secret)):
                if secret[i] == g:
                    pattern[i] = g
        else:
            wrong += 1
        words = [w for w in words if fits(w, pattern, guessed)]
        hit = "hit " if g in secret else "miss"
        print(f"  {g} {hit} {''.join(pattern)} ({len(words)} left)")
    if '_' in pattern:
        print(f"  lost with {wrong} misses")
    else:
        print(f"  solved with {wrong} misses")
    return wrong


def main():
    dictionary = [
        "apple", "angle", "ankle", "baker", "bacon", "beach", "candy", "cabin",
        "dance", "dairy", "eagle", "early", "fable", "fancy", "grape", "gravy",
        "haste", "hazel", "ivory", "jolly", "kneel", "lemon", "lunar", "magic",
        "maple", "noble", "ocean", "piano", "pilot", "quilt", "robot", "ratio",
        "sable", "tango", "tiger", "unity", "vapor", "waltz", "yacht", "zesty",
    ]
    total = 0
    for secret in ["maple", "waltz", "tiger"]:
        total += play(secret, dictionary)
    print(f"total misses: {total}")


def tests():
    pattern = list("_a__e")
    guessed = list("ae")
    assert fits("maple", pattern, guessed)
    assert not fits("eagle", pattern, guessed)
    assert not fits("pale", pattern, guessed)
    assert best_letter(["abc", "abd", "xyz"], []) == 'a'
    assert best_letter(["abc", "abd", "xyz"], ['a']) == 'b'
    assert play("cat", ["cat", "cot", "bat"]) <= 6


import io, contextlib
with contextlib.redirect_stdout(io.StringIO()):
    tests()
main()
