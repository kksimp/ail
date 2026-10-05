from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import TypeVar

T = TypeVar("T")


class Shape(ABC):
    @abstractmethod
    def area(self) -> int: ...

    @abstractmethod
    def name(self) -> str: ...

    def describe(self) -> str:
        return f"{self.name()} with area {self.area()}"


@dataclass
class Circle(Shape):
    r: int

    def area(self) -> int:
        return 3 * self.r * self.r

    def name(self) -> str:
        return "circle"

    def __str__(self) -> str:
        return f"Circle(r: {self.r})"


@dataclass
class Rect(Shape):
    w: int
    h: int

    def area(self) -> int:
        return self.w * self.h

    def name(self) -> str:
        return "square" if self.w == self.h else "rectangle"


@dataclass(order=True)
class Size:
    area: int
    name: str

    def __str__(self) -> str:
        return f'Size(area: {self.area}, name: "{self.name}")'


@dataclass
class Drawing:
    title: str
    shapes: list[Shape] = field(default_factory=list)

    def add(self, s: Shape) -> None:
        self.shapes.append(s)

    def biggest(self) -> Size | None:
        if not self.shapes:
            return None
        best = Size(0, "")
        for s in self.shapes:
            size = Size(s.area(), s.name())
            if size > best:
                best = size
        return best


def largest(items: list[T]) -> T:
    best = items[0]
    for x in items:
        if x > best:
            best = x
    return best


def main() -> None:
    d = Drawing("plan", [Circle(2), Rect(3, 4)])
    d.add(Rect(4, 4))
    for s in d.shapes:
        print(s.describe())
    if (big := d.biggest()) is not None:
        print(f"biggest: {big}")
    print(largest([3, 9, 4]))
    print(largest(["pear", "fig", "apple"]))
    print("true" if Rect(2, 3) == Rect(2, 3) else "false")
    print(d.shapes[0])


main()
