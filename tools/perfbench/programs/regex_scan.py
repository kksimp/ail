# Python twin of regex_scan.ail (CPython 3, `re`, one thread).
import re
import sys

WORDS = ["the", "of", "and", "to", "in", "fox", "jumps", "lazy", "dog", "river", "stone", "moss"]


def line(seed):
    parts = []
    for _ in range(6):
        seed = (seed * 3877 + 29573) % 139968
        t = seed // 16 % 8
        if t == 0:
            parts.append(f"user{seed % 97}@host{seed % 13}.com")
        elif t == 1:
            parts.append(f"id={seed % 1000}")
        elif t == 2:
            parts.append(f"{seed % 100000}")
        elif t == 3:
            parts.append(f"val={seed % 50}")
        else:
            parts.append(WORDS[seed % len(WORDS)])
    return " ".join(parts), seed


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    digits = re.compile("[0-9]+")
    mail = re.compile(r"[a-z0-9]+@[a-z0-9]+\.com")
    pair = re.compile("[a-z]+=[0-9]+")
    seed = 7
    runs = digit_chars = mails = pairs = replaced = 0
    for _ in range(n):
        text, seed = line(seed)
        for m in digits.finditer(text):
            runs += 1
            digit_chars += m.end() - m.start()
        mails += len(mail.findall(text))
        pairs += len(pair.findall(text))
        replaced += len(digits.sub("#", text))
    print(f"digit runs {runs}")
    print(f"digit chars {digit_chars}")
    print(f"addresses {mails}")
    print(f"pairs {pairs}")
    print(f"replaced length {replaced}")


main()
