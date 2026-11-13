def times_small(digits, factor):
    carry = 0
    for i in range(len(digits)):
        v = digits[i] * factor + carry
        digits[i] = v % 10
        carry = v // 10
    while carry > 0:
        digits.append(carry % 10)
        carry = carry // 10


def factorial_digits(n):
    digits = [1]
    for k in range(2, n + 1):
        times_small(digits, k)
    return digits


def power_digits(base, exp):
    digits = [1]
    for _ in range(exp):
        times_small(digits, base)
    return digits


def as_text(digits):
    return "".join(str(d) for d in reversed(digits))


def digit_sum(digits):
    total = 0
    for d in digits:
        total += d
    return total


def trailing_zeros(digits):
    count = 0
    for d in digits:
        if d != 0:
            break
        count += 1
    return count


def report(n):
    digits = factorial_digits(n)
    text = as_text(digits)
    print(f"{n}! has {len(digits)} digits, digit sum {digit_sum(digits)}, {trailing_zeros(digits)} trailing zeros")
    if len(digits) <= 30:
        print(f"  = {text}")
    else:
        print(f"  starts {text[0:12]}...")


def main():
    for n in [5, 10, 20, 25, 50, 100, 250]:
        report(n)
    two = power_digits(2, 100)
    print(f"2^100 = {as_text(two)}")
    print(f"digit sum of 2^100: {digit_sum(two)}")
    small = factorial_digits(0)
    print(f"0! = {as_text(small)}")


main()
