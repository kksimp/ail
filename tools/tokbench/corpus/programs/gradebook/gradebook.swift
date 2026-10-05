enum Grade {
    case a, b, c, f

    var label: String {
        switch self {
        case .a: return "excellent"
        case .b: return "good"
        case .c: return "passing"
        case .f: return "failing"
        }
    }
}

struct Student {
    let name: String
    let score: Int

    var grade: Grade {
        if score >= 90 {
            return .a
        } else if score >= 80 {
            return .b
        } else if score >= 70 {
            return .c
        } else {
            return .f
        }
    }
}

let students = [
    Student(name: "Ada", score: 95),
    Student(name: "Linus", score: 82),
    Student(name: "Grace", score: 67),
]
var total = 0
for s in students {
    print("\(s.name): \(s.score) (\(s.grade.label))")
    total += s.score
}
print("class average: \(total / students.count)")
