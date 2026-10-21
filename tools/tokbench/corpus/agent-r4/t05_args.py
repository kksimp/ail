import argparse
p = argparse.ArgumentParser(description="demo")
p.add_argument("input")
p.add_argument("-v", "--verbose", action="store_true")
p.add_argument("-o", "--out", default="a.txt")
a = p.parse_args()
print(a.verbose, a.out, a.input)
