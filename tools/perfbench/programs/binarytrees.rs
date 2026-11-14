// Binary trees: the same arena build and check as binarytrees.ail.
struct Arena {
    left: Vec<i32>,
    right: Vec<i32>,
    used: usize,
}

impl Arena {
    fn make(&mut self, depth: i32) -> i32 {
        let node = self.used;
        self.used += 1;
        self.left[node] = -1;
        self.right[node] = -1;
        if depth > 0 {
            let l = self.make(depth - 1);
            let r = self.make(depth - 1);
            self.left[node] = l;
            self.right[node] = r;
        }
        node as i32
    }

    fn check(&self, node: i32) -> i64 {
        let n = node as usize;
        if self.left[n] < 0 {
            return 1;
        }
        1 + self.check(self.left[n]) + self.check(self.right[n])
    }
}

fn main() {
    let max_depth: i32 = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(12);
    let min_depth = 4;
    let cap = 1usize << (max_depth + 2);
    let mut arena = Arena { left: vec![0; cap], right: vec![0; cap], used: 0 };
    let mut d = min_depth;
    while d <= max_depth {
        let iters: i64 = 1i64 << (max_depth - d + min_depth);
        let mut total = 0i64;
        for _ in 0..iters {
            arena.used = 0;
            let root = arena.make(d);
            total += arena.check(root);
        }
        println!("{} trees of depth {} check: {}", iters, d, total);
        d += 2;
    }
}
