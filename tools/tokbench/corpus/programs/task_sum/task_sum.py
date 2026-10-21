from concurrent.futures import ThreadPoolExecutor


def sum_range(lo, hi):
    return sum(range(lo, hi))


def main():
    with ThreadPoolExecutor() as pool:
        parts = [pool.submit(sum_range, lo, hi) for lo, hi in [(0, 250), (250, 500), (500, 750), (750, 1000)]]
        total = sum(p.result() for p in parts)
    print(f"sum of 0..1000 in four tasks: {total}")
    print(f"matches the formula: {str(total == 999 * 1000 // 2).lower()}")


if __name__ == "__main__":
    main()
