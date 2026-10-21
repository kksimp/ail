total = 0
for i in range(300000):
    total += i % 13
    print(f"row {i}: value {i * 7 % 1000} sum {total}")
