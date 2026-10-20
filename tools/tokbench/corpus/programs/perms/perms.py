BITS = [256, 128, 64, 32, 16, 8, 4, 2, 1]
MARKS = "rwxrwxrwx"
FILES = [("notes.txt", 0o644), ("run.sh", 0o755), ("secret.key", 0o600), ("shared", 0o775)]


def has(mode, bit):
    return mode & bit != 0


def symbolic(mode):
    return "".join(MARKS[i] if has(mode, BITS[i]) else "-" for i in range(9))


def main():
    for name, mode in FILES:
        owner_write = "yes" if has(mode, 128) else "no"
        other_read = "yes" if has(mode, 4) else "no"
        print(f"{name:<12}{symbolic(mode)}  owner write: {owner_write}, others read: {other_read}")
    owner = sum(1 for bit in BITS[:3] if has(0o600, bit))
    print(f"mode 0o600 has {owner} owner bits")


if __name__ == "__main__":
    main()
