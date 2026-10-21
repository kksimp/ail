import os
n = 0
for root, _, files in os.walk("src"):
    n += sum(1 for f in files if f.endswith(".py"))
print(n)
