def normalize(text):
    return [c for c in text.lower() if c.isalnum()]


def is_palindrome(text):
    cs = normalize(text)
    return cs == cs[::-1]


def expand(cs, lo, hi):
    a = lo
    b = hi
    while a >= 0 and b < len(cs) and cs[a] == cs[b]:
        a -= 1
        b += 1
    return b - a - 1


def longest(text):
    cs = list(text)
    best_start = 0
    best_len = 0
    for i in range(len(cs)):
        odd = expand(cs, i, i)
        even = expand(cs, i, i + 1)
        if odd > best_len:
            best_len = odd
            best_start = i - (odd - 1) // 2
        if even > best_len:
            best_len = even
            best_start = i - even // 2 + 1
    return "".join(cs[best_start:best_start + best_len])


def count_all(text):
    cs = list(text)
    total = 0
    for i in range(len(cs)):
        total += (expand(cs, i, i) + 1) // 2
        total += expand(cs, i, i + 1) // 2
    return total


def digits_palindrome(n):
    return is_palindrome(str(n))


def main():
    phrases = [
        "A man, a plan, a canal: Panama",
        "Was it a car or a cat I saw?",
        "No lemon, no melon",
        "Hello, World",
        "",
    ]
    for p in phrases:
        print("%-32s %s" % (p, "true" if is_palindrome(p) else "false"))
    for s in ["babad", "cbbd", "forgeeksskeegfor", "abacdfgdcaba", "x", "abcde"]:
        print("%s: longest=%s count=%d" % (s, longest(s), count_all(s)))
    found = []
    for n in range(100, 400):
        if digits_palindrome(n) and n % 11 == 0:
            found.append(n)
    print("palindromes divisible by 11 in 100..399: [%s]" % ", ".join(str(x) for x in found))


main()
