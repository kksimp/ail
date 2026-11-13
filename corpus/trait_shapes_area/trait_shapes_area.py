class Circle:
    def __init__(self, r):
        self.r = r

    def area(self):
        return 3 * self.r * self.r

    def perimeter(self):
        return 6 * self.r

    def name(self):
        return "circle"

    def describe(self):
        return f"{self.name()}: area {self.area()}, perimeter {self.perimeter()}"


class Rect(Circle):
    def __init__(self, w, h):
        self.w = w
        self.h = h

    def area(self):
        return self.w * self.h

    def perimeter(self):
        return 2 * (self.w + self.h)

    def name(self):
        return "square" if self.w == self.h else "rect"


class RightTriangle(Circle):
    def __init__(self, a, b, c):
        self.a = a
        self.b = b
        self.c = c

    def area(self):
        return self.a * self.b // 2

    def perimeter(self):
        return self.a + self.b + self.c

    def name(self):
        return "triangle"

    def describe(self):
        return f"triangle {self.a}-{self.b}-{self.c} with area {self.area()}"


shapes = [Circle(4), Rect(6, 3), RightTriangle(6, 8, 10), Rect(5, 5), Circle(1)]
for s in shapes:
    print(s.describe())
print(f"total area: {sum(s.area() for s in shapes)}")
ranked = sorted((s.area(), s.name()) for s in shapes)[::-1]
for place, (area, name) in enumerate(ranked, 1):
    print(f"{place}. {name} {area}")
