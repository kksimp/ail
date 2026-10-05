use std::fmt;

trait Shape: fmt::Display {
    fn area(&self) -> i64;
    fn name(&self) -> String;
    fn describe(&self) -> String {
        format!("{} with area {}", self.name(), self.area())
    }
}

struct Circle {
    r: i64,
}

#[derive(PartialEq)]
struct Rect {
    w: i64,
    h: i64,
}

impl Shape for Circle {
    fn area(&self) -> i64 {
        3 * self.r * self.r
    }
    fn name(&self) -> String {
        "circle".to_string()
    }
}

impl fmt::Display for Circle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Circle(r: {})", self.r)
    }
}

impl Shape for Rect {
    fn area(&self) -> i64 {
        self.w * self.h
    }
    fn name(&self) -> String {
        if self.w == self.h { "square" } else { "rectangle" }.to_string()
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Rect(w: {}, h: {})", self.w, self.h)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Size {
    area: i64,
    name: String,
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Size(area: {}, name: {:?})", self.area, self.name)
    }
}

struct Drawing {
    title: String,
    shapes: Vec<Box<dyn Shape>>,
}

impl Drawing {
    fn add(&mut self, s: Box<dyn Shape>) {
        self.shapes.push(s);
    }

    fn biggest(&self) -> Option<Size> {
        if self.shapes.is_empty() {
            return None;
        }
        let mut best = Size { area: 0, name: String::new() };
        for s in &self.shapes {
            let size = Size { area: s.area(), name: s.name() };
            if size > best {
                best = size;
            }
        }
        Some(best)
    }
}

fn largest<T: Ord + Clone>(items: &[T]) -> T {
    let mut best = items[0].clone();
    for x in items {
        if *x > best {
            best = x.clone();
        }
    }
    best
}

fn main() {
    let mut d = Drawing {
        title: "plan".to_string(),
        shapes: vec![Box::new(Circle { r: 2 }), Box::new(Rect { w: 3, h: 4 })],
    };
    d.add(Box::new(Rect { w: 4, h: 4 }));
    for s in &d.shapes {
        println!("{}", s.describe());
    }
    if let Some(big) = d.biggest() {
        println!("biggest: {big}");
    }
    println!("{}", largest(&[3, 9, 4]));
    println!("{}", largest(&["pear", "fig", "apple"]));
    println!("{}", Rect { w: 2, h: 3 } == Rect { w: 2, h: 3 });
    println!("{}", d.shapes[0]);
}
