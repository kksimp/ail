def signature(word):
    return "".join(sorted(word.lower()))


def group_words(words):
    groups = {}
    seen = set()
    for w in words:
        lower = w.lower()
        if lower not in seen:
            seen.add(lower)
            groups.setdefault(signature(lower), []).append(lower)
    return groups


def is_anagram(a, b):
    return signature(a.replace(" ", "")) == signature(b.replace(" ", ""))


def ranked(groups):
    keys = [k for k in sorted(groups) if len(groups[k]) > 1]
    return sorted(keys, key=lambda k: -len(groups[k]))


def main():
    text = "listen silent enlist google gogole inlets stone notes onset tones steno apple pleap banana nabana cat act tac dog god odg bird Listen"
    words = text.split()
    groups = group_words(words)
    print("words: %d, signatures: %d" % (len(words), len(groups)))
    for k in ranked(groups):
        members = sorted(groups[k])
        print("%-8s %d  %s" % (k, len(members), ", ".join(members)))
    singles = len(groups) - len(ranked(groups))
    print("without anagrams: %d" % singles)
    print("true" if is_anagram("Dormitory", "dirty room") else "false")
    print("true" if is_anagram("hello", "world") else "false")


main()
