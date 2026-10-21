people = [("bo", 30), ("al", 25), ("cy", 30)]
for name, age in sorted(people, key=lambda p: (-p[1], p[0])):
    print(name, age)
