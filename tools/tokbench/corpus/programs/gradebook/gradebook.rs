enum Grade {
    A,
    B,
    C,
    F,
}

impl Grade {
    fn label(self) -> &'static str {
        match self {
            Grade::A => "excellent",
            Grade::B => "good",
            Grade::C => "passing",
            Grade::F => "failing",
        }
    }
}

struct Student {
    name: String,
    score: i64,
}

impl Student {
    fn grade(&self) -> Grade {
        if self.score >= 90 {
            Grade::A
        } else if self.score >= 80 {
            Grade::B
        } else if self.score >= 70 {
            Grade::C
        } else {
            Grade::F
        }
    }
}

fn main() {
    let students = vec![
        Student { name: "Ada".to_string(), score: 95 },
        Student { name: "Linus".to_string(), score: 82 },
        Student { name: "Grace".to_string(), score: 67 },
    ];
    let mut total = 0;
    for s in &students {
        println!("{}: {} ({})", s.name, s.score, s.grade().label());
        total += s.score;
    }
    println!("class average: {}", total / students.len() as i64);
}
