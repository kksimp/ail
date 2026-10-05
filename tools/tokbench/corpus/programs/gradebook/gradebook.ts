enum Grade {
  A,
  B,
  C,
  F,
}

function label(g: Grade): string {
  switch (g) {
    case Grade.A:
      return "excellent";
    case Grade.B:
      return "good";
    case Grade.C:
      return "passing";
    case Grade.F:
      return "failing";
  }
}

class Student {
  constructor(
    public name: string,
    public score: number,
  ) {}

  grade(): Grade {
    if (this.score >= 90) {
      return Grade.A;
    } else if (this.score >= 80) {
      return Grade.B;
    } else if (this.score >= 70) {
      return Grade.C;
    } else {
      return Grade.F;
    }
  }
}

const students = [
  new Student("Ada", 95),
  new Student("Linus", 82),
  new Student("Grace", 67),
];
let total = 0;
for (const s of students) {
  console.log(`${s.name}: ${s.score} (${label(s.grade())})`);
  total += s.score;
}
console.log(`class average: ${Math.trunc(total / students.length)}`);
