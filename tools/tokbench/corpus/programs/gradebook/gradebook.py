from dataclasses import dataclass
from enum import Enum, auto


class Grade(Enum):
    A = auto()
    B = auto()
    C = auto()
    F = auto()

    def label(self) -> str:
        match self:
            case Grade.A:
                return "excellent"
            case Grade.B:
                return "good"
            case Grade.C:
                return "passing"
            case Grade.F:
                return "failing"


@dataclass
class Student:
    name: str
    score: int

    def grade(self) -> Grade:
        if self.score >= 90:
            return Grade.A
        elif self.score >= 80:
            return Grade.B
        elif self.score >= 70:
            return Grade.C
        else:
            return Grade.F


students = [
    Student(name="Ada", score=95),
    Student(name="Linus", score=82),
    Student(name="Grace", score=67),
]
total = 0
for s in students:
    print(f"{s.name}: {s.score} ({s.grade().label()})")
    total += s.score
print(f"class average: {total // len(students)}")
